extern crate embed_manifest;

fn main() {
    // Only embed a Windows manifest when targeting Windows.
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_manifest::embed_manifest(embed_manifest::new_manifest("FocusClient"))
            .expect("unable to embed manifest");
        println!("cargo:rerun-if-changed=manifest.xml");
    }
}
