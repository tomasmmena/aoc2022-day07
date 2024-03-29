use std::fs;
use std::env;
use std::io::{self, BufRead};

use std::cell::RefCell;
use std::rc::Rc;
use std::usize;


trait Node {
    fn get_name(&self) -> &str;
    fn show(&self, level: usize);
}

struct MyDir {
    name: String,
    subdirectories: Vec<Rc<RefCell<MyDir>>>,
    files: Vec<Rc<RefCell<MyFile>>>,
    parent: Option<Rc<RefCell<MyDir>>>
}

impl MyDir {

    fn find(&self, name: &str) -> Option<Rc<RefCell<MyDir>>> {
        match name {
            ".." => {
                if let Some(parent) = &self.parent {
                    return Some(parent.clone())
                }
                None
            },
            "." => None,
            _ => {
                for node in self.subdirectories.iter() {

                    let borrowed = node.borrow();
                    if borrowed.name == name {
                        return Some(node.clone())
                    }
                }
                None
            }
        }
    }

    fn append_subdir(&mut self, new_node: Rc<RefCell<MyDir>>) {
        self.subdirectories.push(new_node);
    }

    fn append_file(&mut self, new_node: Rc<RefCell<MyFile>>) {
        self.files.push(new_node);
    }


    fn get_size(&self) -> usize {
        self.files.iter().map(|f| f.borrow().size).sum::<usize>() +
        self.subdirectories.iter().map(|sd| sd.borrow().get_size()).sum::<usize>()
    }

    fn get_sizes(&self) -> Vec<(String, usize)> {
        let mut starter_list: Vec<(String, usize)> = vec![];
        let mut own_size: usize = self.files.iter().map(|f| f.borrow().size).sum::<usize>();
        for sd in self.subdirectories.iter() {
            let iterator: Vec<(String, usize)> = {
                sd.borrow().get_sizes()
            };
            // there should always be at least one node for the subdir itself
            // it should be the only one considered for the size calculation
            // as it is the only direct child
            own_size += iterator.last().unwrap().1;
            starter_list.extend(iterator.into_iter().to_owned());
        }
        starter_list.push((String::from(&self.name), own_size));

        starter_list
    }
}


impl Node for MyDir {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn show(&self, level: usize) {
        for _ in 0..level {
            print!("  ");
        }
        println!("- {} (dir)", self.name);
        for node in self.subdirectories.iter() {
            node.borrow().show(level + 1);
        }
        for node in self.files.iter() {
            node.borrow().show(level + 1);
        }
    }

}

struct MyFile {
    name: String,
    size: usize,
    parent: Rc<RefCell<MyDir>>
}

impl Node for MyFile {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn show(&self, level: usize) {
        for _ in 0..level {
            print!("  ");
        }
        println!("- {} (file, size={})", self.name, self.size);
    }
}



fn main() {
    let path = env::args().nth(1).expect("No file path was specified.");

    let data = io::BufReader::new(
        fs::File::open(path).expect("Could not open file!")
    ).lines().skip(1);

    // start at the root
    let mut current_node: Rc<RefCell<MyDir>> = Rc::new(
        RefCell::new(
            MyDir {
                name: "/".to_string(),
                subdirectories: vec![],
                files: vec![],
                parent: None
            }
        )
    );

    for line in data.into_iter() {
        let l = line.expect("Could not read line!");
        let (starter, other) = l.split_at(
            l.find(" ").expect(format!("Invalid line \"{}\"!", l).as_str())
        );
        match starter {
            "$" => {  // command, either cd or ls
                let (cmd, params) = other[1..other.len()].split_at(
                    other[1..other.len()].find(" ").unwrap_or(0)
                );
                match cmd {
                    "cd" => {
                        let next_node = {
                            current_node.borrow().find(&params[1..params.len()]).expect("invalid cd!")
                        };
                        current_node = next_node.clone();
                    },
                    _ => {}
                }
            },
            "dir" => {  // a directory that's in the current node
                current_node.borrow_mut().append_subdir(
                    Rc::new(RefCell::new(
                        MyDir {
                            name: other[1..other.len()].to_string(),
                            subdirectories: vec![],
                            files: vec![],
                            parent: Some(current_node.clone())
                        }
                    ))
                )
            },
            _ => { // a number or a bad starter
                let size: usize = starter.parse().expect("Invalid start");
                current_node.borrow_mut().append_file(
                    Rc::new(RefCell::new(
                        MyFile { 
                            name: other[1..other.len()].to_string(), 
                            size: size, 
                            parent: current_node.clone() 
                        }
                    ))
                )
            }
        }
    }
    
    
    loop {  // go up to the root
        let parent = {
            match &current_node.borrow().parent {
                Some(parent_node) => Some(parent_node.clone()),
                None => None
            }
        };
        if let Some(parent_node) = parent {
            current_node = parent_node.clone();
        } else {
            break;
        }

    }

    // current_node.borrow().show(0);
    {
        let cn = current_node.borrow();
        let total_size: usize = cn
            .get_sizes()
            .into_iter()
            .map(|(_, size)| size)
            .filter(|size| size <= &100000)
            .sum();
        println!("Total size of dirs of at most 100000: {}", total_size);

        let threshold = 30_000_000 - (70_000_000 - cn.get_size());
        println!("Threshold = {}", threshold);
        let mut sizes: Vec<(String, usize)> = cn
            .get_sizes()
            .into_iter()
            .filter(|(_, size)| size > &threshold)
            .collect();
        sizes.sort_by_key(|(_, size)| size.clone());
        let smallest = sizes.first().expect("No acceptable candidates!");
        println!("Smallest directory over threshold is sized: {}", smallest.1);
        
    }

}


#[test]
fn test_simple_fs() {
    let root = Rc::new(
        RefCell::new(
            MyDir {
                name: "/".to_string(),
                subdirectories: vec![],
                files: vec![],
                parent: None
            }
        )
    );

    root.borrow_mut().files.push(
        Rc::new(
            RefCell::new(
                MyFile{
                    name: "afile.exe".to_string(),
                    size: 1000000,
                    parent: Rc::clone(&root)
                }
            )
        )
    );
 
    root.borrow_mut().files.push(
        Rc::new(
            RefCell::new(
                MyFile{
                    name: "bfile.exe".to_string(),
                    size: 2000000,
                    parent: Rc::clone(&root)
                }
            )
        )
    );

    let subdir = Rc::new(
        RefCell::new(
            MyDir {
                name: "adir".to_string(),
                subdirectories: vec![],
                files: vec![],
                parent: Some(root.clone())
            }
        )
    );

 
    subdir.borrow_mut().files.push(
        Rc::new(
            RefCell::new(
                MyFile{
                    name: "cfile.exe".to_string(),
                    size: 1000000,
                    parent: Rc::clone(&subdir)
                }
            )
        )
    );

    root.borrow_mut().subdirectories.push(subdir.clone());


    root.borrow().show(0);
    assert_eq!(root.borrow().get_size(), 4000000);
    assert_eq!(subdir.borrow().get_size(), 1000000);

    let sizes = vec![
        ("adir".to_string(), 1000000 as usize),
        ("/".to_string(), 4000000 as usize)
    ];
    assert_eq!(root.borrow().get_sizes(), sizes);
}
