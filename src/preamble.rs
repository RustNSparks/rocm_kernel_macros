use quote::quote;

pub fn preamble() -> proc_macro::TokenStream {
    quote! {
        #![no_std]
        #![feature(abi_gpu_kernel)]
        #![feature(core_intrinsics, link_llvm_intrinsics)]

        #[panic_handler]
        fn panic(_: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        unsafe extern "C" {
            #[link_name = "llvm.amdgcn.workitem.id.x"]
            pub fn workitem_id_x() -> u32;
            #[link_name = "llvm.amdgcn.workitem.id.y"]
            pub fn workitem_id_y() -> u32;
            #[link_name = "llvm.amdgcn.workitem.id.z"]
            pub fn workitem_id_z() -> u32;

            #[link_name = "llvm.amdgcn.workgroup.id.x"]
            pub fn workgroup_id_x() -> u32;
            #[link_name = "llvm.amdgcn.workgroup.id.y"]
            pub fn workgroup_id_y() -> u32;
            #[link_name = "llvm.amdgcn.workgroup.id.z"]
            pub fn workgroup_id_z() -> u32;
        }
    }
    .into()
}

pub fn dummy_preamble() -> proc_macro::TokenStream {
    quote! {
        pub unsafe fn workitem_id_x() -> u32 {0}
        pub unsafe fn workitem_id_y() -> u32 {0}
        pub unsafe fn workitem_id_z() -> u32 {0}
        pub unsafe fn workgroup_id_x() -> u32 {0}
        pub unsafe fn workgroup_id_y() -> u32 {0}
        pub unsafe fn workgroup_id_z() -> u32 {0}
    }
    .into()
}
