# Conversion code

A big part of rustc's driver is the conversion between rustc's HIR representation and marker's
AST representation. This document outlines the conversion module and defines rough guidelines
how to structure conversion code in rustc.

All conversion code is part of a `ConversionContext` which holds a reference to rustc's `TyCtxt`
and the `Storage` object, for memory allocation. The `ConversionContext` provides simple access
to these shared instances and makes it simpler to deal with lifetimes. It also groups functions
together based on the conversion direction:

* `MarkerConversionContext` for rustc to marker conversions
* `RustcConversionContext` for marker to rustc conversions

Information only used by one conversion direction is stored in the related context. An example
for this is the node cache, which stores converted nodes. The cache is only needed for the rustc
to marker direction. Information required by both converters is stored in the shared `Storage` object.
An example is `SpanSourceInfo` required to create the API spans and convert them back to rustc's
representation.

Structs used by both conversion directions, like ID layouts, are stored inside the `common`
submodule. The conversion context instances should be independent and in theory don't know
of the other one.

Conversion functions generally use the `to_` prefix, followed by the target object. `to_symbol_id`
for example, takes a symbol and converts it to a `SymbolId` the direction is implied by the context
that the function is defined in. For common conversion targets `_from_` can be added to
define the conversion source. An example for this is `to_callable_data_from_func_sig` and
`to_callable_data_from_func_decl`. For simple cases, `impl Into<Type>` can be used to make the
interface more ergonomic.

The `RustcContext` finally pulls it all together, delegating conversions to the converters as needed.
