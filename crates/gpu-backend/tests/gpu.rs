//! GPU backend tests. These require a real GPU adapter and are skipped unless
//! `SPECTRA_RUN_GPU=1` is set in the environment (CI runners typically have no
//! GPU, so they are ignored by default).

use spectra_gpu_backend::GpuContext;

#[tokio::test]
#[ignore = "requires a GPU adapter; set SPECTRA_RUN_GPU=1"]
async fn gpu_context_initializes() {
    if std::env::var("SPECTRA_RUN_GPU").is_err() {
        eprintln!("skipping: SPECTRA_RUN_GPU not set");
        return;
    }
    let _ctx = GpuContext::init_compute().await.expect("init gpu");
    // If we reached here, adapter + device + queue were acquired successfully.
}
