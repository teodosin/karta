/// Test utilities for the vault.
/// Located here are functions for creating mock vaults, mock files and directories, and cleaning up after tests.
#[cfg(test)]
pub mod vault_utils {
    use std::{path::PathBuf, io::Write};

    use crate::vault::KartaVault;

    use directories::ProjectDirs;

    /// Creating a temporary vault for testing.
    /// Pass in a unique name for each test so the vaults don't overlap
    pub fn get_test_vault(test_id: &str) -> KartaVault {
        let project_dirs: ProjectDirs = ProjectDirs::from("com", "Teodosin", "Karta").unwrap();
        let config_dir = project_dirs.config_dir();

        let mut vault_path = config_dir.to_path_buf();
        vault_path.push(test_id);

        // Create the directory
        if !vault_path.exists() {
            std::fs::create_dir(&vault_path).unwrap();
        }
        
        let vault = KartaVault::new(vault_path);
        vault
    }

    /// Deleting the temporary vault
    pub fn cleanup_test_vault(vault: KartaVault){
        let binding = vault.get_vault_folder_name();
        let name = binding.to_str().unwrap();
        let vault = get_test_vault(name.clone());
        let path = vault.get_root_path();

        assert!(path.exists(), "Path does not exist");
        assert!(path.is_dir(), "Path is not a directory");
        assert!(path.file_name().unwrap() == name, "Path does not have the correct name: {:?} compared to {}", path.file_name().unwrap(), name);

        std::fs::remove_dir_all(path).unwrap();
    }

    pub fn create_test_dir(path: PathBuf){
        std::fs::create_dir(path).unwrap();
    }

    pub fn create_test_file_empty_text(path: PathBuf){
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(b"").unwrap();
    }

    pub fn create_test_dir_with_file(path: PathBuf){
        create_test_dir(path.clone());
        create_test_file_empty_text(path.join("test.txt"));
    }

    #[test]
    fn test_vault_is_created_and_cleaned_up(){

        let name = "test_vault_is_created_and_cleaned_up";
        let vault = get_test_vault(name);
        let name = vault.get_vault_folder_name();
        let path = vault.get_root_path();

        assert!(path.exists(), "Path does not exist");
        assert!(path.is_dir(), "Path is not a directory");
        assert!(path.file_name().unwrap() == name, "Path does not have the correct name: {:?} compared to {:?}", path.file_name().unwrap(), name);

        cleanup_test_vault(vault);

        assert!(!path.exists());
    }

    #[test]
    fn test_create_test_dir(){
        let name = "test_create_test_dir";
        let vault = get_test_vault(name);
        let path = vault.get_root_path();

        let test_dir = path.join("test_dir");

        assert!(!test_dir.exists());

        create_test_dir(test_dir.clone());

        assert!(test_dir.exists());
        assert!(test_dir.is_dir());

        cleanup_test_vault(vault);
    }
}