trait CommandTrait {
    fn execute(&self);
    fn rollback(&self);
}

struct Commands {
    commands: Vec<Box<dyn CommandTrait>>,
}

impl Commands {
    fn new() -> Commands {
        Commands {
            commands: Vec::new(),
        }
    }

    fn add_commands(&mut self, command: Box<dyn CommandTrait>) {
        self.commands.push(command);
    }

    fn execute(&self) {
        self.commands.iter().for_each(|command| command.execute());
    }

    fn rollback(&self) {
        self.commands
            .iter()
            .rev()
            .for_each(|command| command.rollback());
    }
}

struct Command1 {
    name: String,
}

impl CommandTrait for Command1 {
    fn execute(&self) {
        println!("{} executed", self.name);
    }

    fn rollback(&self) {
        println!("{} rolled back", self.name);
    }
}

struct Command2 {
    name: String,
}

impl CommandTrait for Command2 {
    fn execute(&self) {
        println!("{} executed", self.name);
    }

    fn rollback(&self) {
        println!("{} rolled back", self.name);
    }
}

/*
   利用Trait对象，可以将不同类型的对象放入同一个容器中，这样就可以对这些对象进行统一的管理。
*/
#[test]
fn test() {
    let mut commands = Commands::new();
    commands.add_commands(Box::new(Command1 {
        name: "Command1".to_string(),
    }));
    commands.add_commands(Box::new(Command2 {
        name: "Command2".to_string(),
    }));
    commands.execute();
    commands.rollback();
}

type FnPtr = fn() -> String;

struct FnPtrCommands {
    execute_commands: Vec<FnPtr>,
    rollback_commands: Vec<FnPtr>,
}

impl FnPtrCommands {
    fn new() -> FnPtrCommands {
        FnPtrCommands {
            execute_commands: Vec::new(),
            rollback_commands: Vec::new(),
        }
    }
    fn add_commands(&mut self, execute_command: FnPtr, rollback_command: FnPtr) {
        self.execute_commands.push(execute_command);
        self.rollback_commands.push(rollback_command);
    }
    fn add_all_commands_from_other_fnptrcommands(&mut self, commands: &mut FnPtrCommands) {
        self.execute_commands.append(&mut commands.execute_commands);
        self.rollback_commands
            .append(&mut commands.rollback_commands);
    }
    fn add_all_commands_from_vec(
        &mut self,
        execute_commands: Vec<FnPtr>,
        rollback_commands: Vec<FnPtr>,
    ) {
        self.execute_commands.append(&mut execute_commands.clone());
        self.rollback_commands
            .append(&mut rollback_commands.clone());
    }
    fn execute(&self) -> Vec<String> {
        self.execute_commands
            .iter()
            .map(|command| (*command)())
            .collect()
    }
    fn rollback(&self) -> Vec<String> {
        self.rollback_commands
            .iter()
            .rev()
            .map(|command| (*command)())
            .collect::<Vec<String>>()
    }
}

#[test]
fn test_fn_ptr_command_pattern() {
    let fn1_execute = || {
        println!("fn1 executed");
        "fn1 executed".to_string()
    };
    let fn1_rollback = || {
        println!("fn1 rollback");
        "fn1 rollback".to_string()
    };
    let fn2_execute = || {
        println!("fn2 executed");
        "fn2 executed".to_string()
    };
    let fn2_rollback = || {
        println!("fn2 rollback");
        "fn2 rollback".to_string()
    };
    let mut fnptrcommands = FnPtrCommands::new();
    fnptrcommands.add_commands(fn1_execute, fn1_rollback);
    fnptrcommands.add_commands(fn2_execute, fn2_rollback);

    fnptrcommands.execute();
    fnptrcommands.rollback();
}
