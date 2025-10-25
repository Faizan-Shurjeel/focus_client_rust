extern crate embed_manifest;

fn main() {
    embed_manifest::embed_manifest(embed_manifest::new_manifest("FocusClient"))
        .expect("unable to embed manifest");
    println!("cargo:rerun-if-changed=manifest.xml");
}