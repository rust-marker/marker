# Lint Crate Security

Marker's lint crates are normal crates compiled into dynamic libraries. The compilation is done with Cargo, which will execute the build script and proc macros used by the lint crates. This means that lint crates can theoretically perform malicious activities during the compilation and checking processes. It's advised to only run lint crates from trusted sources, perform audits of used crates, or use a sandbox.

## Malicious behavior in lint crates

We're sadly unable to review and validate all lint crates. Marker also doesn't host any lint crates itself. Instead, it relies on existing infrastructure like git repositories and crate registries. If you notice malicious behavior in a lint crate, please handle it like you would with any other dependency. It's recommended to take the following steps:

1. Avoid using the lint crate until the issue has been resolved.
2. If the malicious behavior seems to be incidental, contact the author to fix the issue.
3. If the malicious behavior seems to be intentional, report the lint crate according to the policy of the hosting platform.

## Sandboxing

Lint crates are loaded as normal dynamic libraries. Marker sadly doesn't provide any option to sandbox the lint crates. If you want to use an untrusted lint crate, it's recommended that you manually sandbox the entire Marker process.
