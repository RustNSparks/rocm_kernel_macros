# rocm_kernel_macros

Crate for generating subprojects with kernel source written in rust

## Requirements

1. rust nightly
2. rust target: amdgcn-amd-amdhsa
3. rust-src
4. ROCM 6.0 or newer

## Examples 

1. Writing gpu kernels in rust
```rust
// initialize new kernel subproject
amdgpu_kernel_init!();

// mark function that will be copied to kernel src
#[amdgpu_global]
fn kernel(input: *const u32, output: *mut u32) {
    // extract data from pointer by workitem id x using helper function
    let mut num = read_by_workitem_id_x(input);

    num += 4;

    // write data back using helper function
    write_by_workitem_id_x(output, num);
}

// compile and get path to kernel binary
const AMDGPU_KERNEL_BINARY_PATH: &str = amdgpu_kernel_finalize!();
```

2. Running kernel on gpu side using `rocm-rs` (assuming above kernel)
```rust
    // aquire device
    let device = Device::new(0)?;
    device.set_current()?;

    // load kernel binary
    let kernel_path = PathBuf::from(AMDGPU_KERNEL_BINARY_PATH);
    assert!(kernel_path.exists());
    let module = Module::load(kernel_path)?;

    // search for function in module
    let function = unsafe { module.get_function("kernel")? };

    // prepare input and output memory
    let mut in_host: Vec<u32> = vec![0; LEN];
    let mut out_host: Vec<u32> = vec![0; LEN];
     
    // prepare data
    for i in 0..LEN {
        in_host[i] = i as u32;
    }

    let mut input = DeviceMemory::<u32>::new(LEN)?;
    let output = DeviceMemory::<u32>::new(LEN)?;

    input.copy_from_host(&in_host)?;


    // prepare kernel arguments
    let kernel_args = [input.as_kernel_arg(), output.as_kernel_arg()];

    // setup launch arguments
    let grid_dim = Dim3 { x: 2, y: 1, z: 1 };
    let block_dim = Dim3 {
        x: (LEN / 2) as u32,
        y: 1,
        z: 1,
    };

    // launch kernel (grid_dim, block_dim, shared_mem_bytes, stream, args)
    function.launch(grid_dim, block_dim, 0, None, &mut kernel_args.clone())?;
```

3. Running kernel on host side (assuming above kernel, highly not recomended)
```rust

fn main() {
    // prepare input and output
    let input = (0..64).collect::<Vec<_>>();
    let mut output = vec![0; 64];

    for i in 0..64 {
        // set global id variable
        WORKITEM_ID_X.store(i, Ordering::Relaxed);
        
        kernel(input.as_ptr(), output.as_mut_ptr());
    }

    println!("{:?}", output);
}
```
