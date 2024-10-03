fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/factory.proto"], &["."])?;
    tonic_build::configure().compile(
        &["../fr-pmx-registry/proto/registry.proto"],
        &["../fr-pmx-registry/"],
    )?;
    tonic_build::configure().compile(
        &["../fr-pmx-mod-host-proxy/proto/proxy.proto"],
        &["../fr-pmx-mod-host-proxy/"],
    )?;
    tonic_build::configure().compile(
        &["../fr-pipewire-registry/proto/pipewire.proto"],
        &["../fr-pipewire-registry/"],
    )?;
    Ok(())
}
