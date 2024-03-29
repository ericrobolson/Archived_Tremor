use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;
use rayon::prelude::*;
use std::env;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

/// Copy 'res' folder
pub fn copy_res() -> Result<()> {
    // Copy assets
    println!("cargo:rerun-if-changed=src/*");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = vec![];
    paths_to_copy.push("../res/");
    copy_items(&paths_to_copy, out_dir, &copy_options).unwrap();

    Ok(())
}

/// Copy + build SPV shaders
pub fn gfx_shader_build() -> Result<()> {
    println!("cargo:rerun-if-changed=src/*");

    // Collect all shaders recursively within /src/
    let mut shader_paths = vec![];
    shader_paths.extend(glob("./src/**/shaders/*.vert")?);
    shader_paths.extend(glob("./src/**/shaders/*.frag")?);
    shader_paths.extend(glob("./src/**/shaders/*.comp")?);

    // This could be parallelized
    let shaders = shader_paths
        .into_par_iter()
        .map(|glob_result| ShaderData::load(glob_result?))
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>();

    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;

    // This can't be parallelized. The [shaderc::Compiler] is not
    // thread safe. Also, it creates a lot of resources. You could
    // spawn multiple processes to handle this, but it would probably
    // be better just to only compile shaders that have been changed
    // recently.
    for shader in shaders? {
        let compiled = compiler.compile_into_spirv(
            &shader.src,
            shader.kind,
            &shader.src_path.to_str().unwrap(),
            "main",
            None,
        )?;
        write(shader.spv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}

struct ShaderData {
    src: String,
    src_path: PathBuf,
    spv_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension can't be converted to &str")?;
        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };
        let src = read_to_string(src_path.clone())?;

        let file_name = {
            let file_name = src_path.file_name().unwrap();
            file_name
        };

        let output_path = {
            let mut new_path = src_path.clone();
            new_path.pop();
            new_path.push("spv");
            new_path.push(file_name);
            new_path
        };

        let spv_path = output_path.with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}
