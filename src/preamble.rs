use quote::quote;

pub fn preamble() -> String {
    quote! {

    #![no_std]
    #![feature(abi_gpu_kernel)]
    #![feature(core_intrinsics, link_llvm_intrinsics)]

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo) -> ! {
        loop {}
    }
    mod llvm_bindings{

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

    pub fn workitem_id_x() -> u32 {unsafe {llvm_bindings::workitem_id_x()}}
    pub fn workitem_id_y() -> u32 {unsafe {llvm_bindings::workitem_id_y()}}
    pub fn workitem_id_z() -> u32 {unsafe {llvm_bindings::workitem_id_z()}}
    pub fn workgroup_id_x() -> u32 {unsafe {llvm_bindings::workgroup_id_x()}}
    pub fn workgroup_id_y() -> u32 {unsafe {llvm_bindings::workgroup_id_y()}}
    pub fn workgroup_id_z() -> u32 {unsafe {llvm_bindings::workgroup_id_z()}}
    }
    .to_string()
}

pub fn dummy_preamble() -> proc_macro::TokenStream {
    quote! {
        pub fn workitem_id_x() -> u32 {0}
        pub fn workitem_id_y() -> u32 {0}
        pub fn workitem_id_z() -> u32 {0}
        pub fn workgroup_id_x() -> u32 {0}
        pub fn workgroup_id_y() -> u32 {0}
        pub fn workgroup_id_z() -> u32 {0}
    }
    .into()
}
