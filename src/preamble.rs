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

    pub fn read_by_workitem_id_x<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_x() as usize)}}
    pub fn read_by_workitem_id_y<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_y() as usize)}}
    pub fn read_by_workitem_id_z<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_z() as usize)}}
    pub fn read_by_workgroup_id_x<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_x() as usize)}}
    pub fn read_by_workgroup_id_y<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_y() as usize)}}
    pub fn read_by_workgroup_id_z<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_z() as usize)}}

    pub fn write_by_workitem_id_x<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_x() as usize) = value}}
    pub fn write_by_workitem_id_y<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_y() as usize) = value}}
    pub fn write_by_workitem_id_z<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_z() as usize) = value}}
    pub fn write_by_workgroup_id_x<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_x() as usize) = value}}
    pub fn write_by_workgroup_id_y<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_y() as usize) = value}}
    pub fn write_by_workgroup_id_z<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_z() as usize) = value}}

    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preamble_uses_correct_dimensions() {
        let code = preamble();

        let expected = [
            ("read_by_workitem_id_x", "workitem_id_x"),
            ("read_by_workitem_id_y", "workitem_id_y"),
            ("read_by_workitem_id_z", "workitem_id_z"),
            ("read_by_workgroup_id_x", "workgroup_id_x"),
            ("read_by_workgroup_id_y", "workgroup_id_y"),
            ("read_by_workgroup_id_z", "workgroup_id_z"),
            ("write_by_workitem_id_x", "workitem_id_x"),
            ("write_by_workitem_id_y", "workitem_id_y"),
            ("write_by_workitem_id_z", "workitem_id_z"),
            ("write_by_workgroup_id_x", "workgroup_id_x"),
            ("write_by_workgroup_id_y", "workgroup_id_y"),
            ("write_by_workgroup_id_z", "workgroup_id_z"),
        ];

        let normalized: String = code.chars().filter(|c| !c.is_whitespace()).collect();

        for (fn_name, expected_call) in expected {
            let fn_start = normalized
                .find(&format!("fn{fn_name}"))
                .unwrap_or_else(|| panic!("missing function: {fn_name}"));
            let body_end = normalized[fn_start..]
                .find('}')
                .map(|i| fn_start + i)
                .unwrap_or_else(|| panic!("no closing brace for {fn_name}"));
            let body = &normalized[fn_start..body_end];
            let call_with_parens = format!("{expected_call}()");
            assert!(
                body.contains(&call_with_parens),
                "{fn_name} should call {call_with_parens}, but body was: {body}"
            );
        }
    }
}

pub fn dummy_preamble() -> proc_macro::TokenStream {
    quote! {
        static WORKITEM_ID_X: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static WORKITEM_ID_Y: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static WORKITEM_ID_Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static WORKGROUP_ID_X: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static WORKGROUP_ID_Y: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static WORKGROUP_ID_Z: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        pub fn workitem_id_x() -> u32 {WORKITEM_ID_X.load(std::sync::atomic::Ordering::Relaxed)}
        pub fn workitem_id_y() -> u32 {WORKITEM_ID_Y.load(std::sync::atomic::Ordering::Relaxed)}
        pub fn workitem_id_z() -> u32 {WORKITEM_ID_Z.load(std::sync::atomic::Ordering::Relaxed)}
        pub fn workgroup_id_x() -> u32 {WORKGROUP_ID_X.load(std::sync::atomic::Ordering::Relaxed)}
        pub fn workgroup_id_y() -> u32 {WORKGROUP_ID_Y.load(std::sync::atomic::Ordering::Relaxed)}
        pub fn workgroup_id_z() -> u32 {WORKGROUP_ID_Z.load(std::sync::atomic::Ordering::Relaxed)}

        pub fn read_by_workitem_id_x<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_x() as usize)}}
        pub fn read_by_workitem_id_y<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_y() as usize)}}
        pub fn read_by_workitem_id_z<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workitem_id_z() as usize)}}
        pub fn read_by_workgroup_id_x<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_x() as usize)}}
        pub fn read_by_workgroup_id_y<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_y() as usize)}}
        pub fn read_by_workgroup_id_z<T: Clone + Copy>(data: *const T) -> T { unsafe {*data.add(workgroup_id_z() as usize)}}

        pub fn write_by_workitem_id_x<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_x() as usize) = value}}
        pub fn write_by_workitem_id_y<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_y() as usize) = value}}
        pub fn write_by_workitem_id_z<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workitem_id_z() as usize) = value}}
        pub fn write_by_workgroup_id_x<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_x() as usize) = value}}
        pub fn write_by_workgroup_id_y<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_y() as usize) = value}}
        pub fn write_by_workgroup_id_z<T: Clone + Copy>(target: *mut T, value: T) { unsafe {*target.add(workgroup_id_z() as usize) = value}}
    }
    .into()
}
