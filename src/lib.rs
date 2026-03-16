pub mod repo;
pub mod utils;
pub mod object;
pub mod ignore;
pub mod index;

pub mod commands {
    pub mod init;
    pub mod hash_object;
    pub mod cat_file;
    pub mod add;
    pub mod commit;
    pub mod log;
    pub mod status;
    pub mod branch;
    pub mod switch;
}
