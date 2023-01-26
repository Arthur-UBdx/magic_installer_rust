// ---- Tests ---- //

#[cfg(test)]
mod tests {
    #[test]
    fn test_env_path() {
        let path = "%appdata%\\.minecraft";
        let path_var = crate::file_handling::get_env_path(path);
        assert_eq!(&path_var, "C:\\Users\\admin\\AppData\\Roaming\\.minecraft");
    }

    #[test]
    fn test_remove_mods() {
        let config: Config::from(String::from(""), String::form(""))
    }
}