# Driver information

This document contains information required to implement drivers.

## Lifetimes

### Models

Lint-crates officially only know the `'ast` lifetime.
All non-copy items given to lint-crates have this lifetime.
For a lint crate, it feels like the `'ast` lifetime start with
the call of a `check_` function and also ends with it.
Drivers layer their implementation on top of this view.

Rustc's driver uses roughly the following lifetime model:

```
'tcx by Rustc:       |------------------------------------------|
'ast by Driver:         |------------------------------------|
Lint-crates:               |------------------------------|
LintPass functions:           |--------|  |----|  |----|
```

The lint-crates are loaded after the entire AST has been mapped
and will only be dropped after they're unloaded.
This makes `'ast` basically equivalent to `'static` in this model.
However, this API is not only intended for compilers like rustc,
but also potentially language servers.
These have different requirements as the AST has to change.
This means nodes have to be dropped and rechecked again.

The following showcases a model that could be used by rust-analyzer:

```
rust-analyzer:        |------------------------------------|
Lint-crates:             |------------------------------|
'ast for AST nodes:         |----------|  |----------|
LintPass functions:            |-----|      |-----|
```

Notice that here the lint-crates live longer than the AST they're analyzing.

### `'static AstContext` hackery

AST nodes can provide a lot of information about themselves and their context.
Most of this information is not stored in the node itself,
but dynamically loaded using an `AstContext` instance.
In the initial prototype, the `AstContext` was stored in each node.
However, this added an unnecessary reference to every node,
increasing their size and making the driver code more convoluted.
For these reasons, the implementation was changed to use a global instance that is used by all nodes.

This global instance is set by the adapter before any `check_*` function is called inside the lint crate.
The context will be updated once for every tree given to the adapter.

The current implementation imposes the following requirements:

1. The `AstContext` instance has to be valid for the entire time, that it takes to process the AST passed to the adapter.
    This means that the instance as well as the AST has to remain in memory.
    This behavior can implement all models as described above.
2. Lint crates should always be called by the same thread and not be accessed concurrently.
    (The implementation might be able to handle it, but this is not actively tested.)
3. The driver should never call functions on AST nodes that depend on the current `AstContext` instance.
    (In general, there should be no reason to do so.)
