use risc0_build_ethereum::generate_solidity_files;

const SOLIDITY_IMAGE_ID_PATH: &str = "../contracts/src/ImageID.sol";

fn main() {
    risc0_build::embed_methods();

    // Generate Solidity source files for use with Forge.
    let solidity_opts = risc0_build_ethereum::Options::default()
        .with_image_id_sol_path(SOLIDITY_IMAGE_ID_PATH);
}

