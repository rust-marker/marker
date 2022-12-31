# Type guidelines

This file contains guidelines for API types and the reasoning behind these:

* API types should all have `#[repr(C)]`
* API enums should all have a `#[non_exhaustive]` attribute
* Slices provided and stored by the API should contain objects and not references to objects:
    * Prefer `&[Parameter]` over `&[&Parameter]`.
    This reduces the number of lifetimes and helps the driver creation a bit.
    The borrow checker will enforce, that the items in the slice are never moved unless they're `Copy`.
    This also means that we should be careful which items we give `Copy`
