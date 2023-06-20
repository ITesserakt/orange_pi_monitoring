use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let iter = std::fs::read_dir("proto")?.filter_map(|x| x.ok());

    for entry in iter {
        if let Some(file) = entry.file_name().to_str() {
            if file.ends_with(".proto") {
                tonic_build::configure()
                    .build_client(true)
                    .build_server(true)
                    .build_transport(false)
                    .compile(&[file], &[entry.path().parent().unwrap()])?;
            }
        }
    }

    Ok(())
}
