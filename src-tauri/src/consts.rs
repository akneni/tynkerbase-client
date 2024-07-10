const fn get_app_data_path() -> &'static str {
    #[cfg(target_os = "linux")]
    return "~/.local/share/tynkerbase/";
    
    #[cfg(target_os = "windows")]
    return "C:\\Users\\username\\AppData\\Local\\TynkerBase";

    #[allow(unreachable_code)]
    ""
}


pub const APP_DATA: &str = get_app_data_path();
pub const AUTH_ENDPOINT: &str = "https://tynkerbase-server.shuttleapp.rs";
