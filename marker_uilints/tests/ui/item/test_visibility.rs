fn test_vis_private() {}

pub fn test_vis_public() {}

mod module {
    pub(crate) fn test_vis_pub_crate() {}

    pub(super) fn test_vis_pub_super_crate_root() {}

    mod nested {
        pub(super) fn test_vis_pub_super() {}

        pub(in crate::module) fn test_vis_pub_in_path() {}
    }
}

fn main() {}
