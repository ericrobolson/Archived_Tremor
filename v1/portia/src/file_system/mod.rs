mod asset_loader;
pub use asset_loader::{Asset, AssetState, LoadableAsset};

pub struct FileSystem {}

impl FileSystem {
    fn res_path() -> String {
        "res".into()
    }

    pub fn res_dir() -> std::path::PathBuf {
        use std::env;
        env::current_dir().unwrap().join(Self::res_path())
    }

    pub fn get_file(file_name: &'static str) -> std::path::PathBuf {
        use std::{env, fs};
        use walkdir::WalkDir;

        let mut path = env::current_dir().unwrap();
        path.push(Self::res_path());

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let entry_name = match entry.file_name().to_str() {
                Some(s) => s,
                None => "",
            };

            if file_name == entry_name {
                return entry.into_path();
            }
        }

        panic!("Unable to find file with name '{}'!", file_name);
    }
}
