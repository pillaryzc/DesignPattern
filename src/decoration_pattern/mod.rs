mod file;
mod folder;

pub use file::File;
pub use folder::Folder;

pub trait Component {
    fn search(&self, keyword: &str);
}

#[test]
fn test_decoration_pattern(){
    let file1 = File::new("file1");
    let file2 = File::new("file2");
    let mut folder = Folder::new("Folder 1");
    folder.add(file1);
    folder.add(file2);

    let mut folder2 = Folder::new("Folder 2");
    let file3 = File::new("file3");
    folder2.add(file3);
    folder2.add(folder);

    folder2.search("rose");
}