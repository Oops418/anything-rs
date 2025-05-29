use gpui_component::Icon;
use material_icon_embed_rs::material_icon_file::MaterialIconFile;
use material_icon_embed_rs::material_icon_folder::MaterialIconFolder;

pub struct FileIcon(pub MaterialIconFile);

pub struct FolderIcon(pub MaterialIconFolder);

impl From<FileIcon> for Icon {
    fn from(val: FileIcon) -> Self {
        Icon::default().path(val.0.path())
    }
}

impl From<FolderIcon> for Icon {
    fn from(val: FolderIcon) -> Self {
        Icon::default().path(val.0.path())
    }
}

impl From<Option<MaterialIconFile>> for FileIcon {
    fn from(val: Option<MaterialIconFile>) -> Self {
        match val {
            Some(icon_file) => FileIcon(icon_file),
            None => FileIcon(MaterialIconFile::Document),
        }
    }
}

impl From<Option<MaterialIconFolder>> for FolderIcon {
    fn from(val: Option<MaterialIconFolder>) -> Self {
        match val {
            Some(icon_folder) => FolderIcon(icon_folder),
            None => FolderIcon(MaterialIconFolder::FolderDocs),
        }
    }
}
