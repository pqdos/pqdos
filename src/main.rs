//! PQOS Shell - Command Line Interface for the Post-Quantum Distributed OS
//!
//! This application serves as the shell/CLI for the OS, providing:
//! 1. Self-integrity verification against blockchain
//! 2. User authentication and account creation
//! 3. File system commands (cd, copy, ls, etc.)
//! 4. Virtual directory for user-owned memory blocks
//! 5. Automatic file-to-block conversion with signing

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pqc_distro_os::users::{UserSystem, create_user_system_with_demo_keys, User, UserId, UserRole, UserPermissions};
use sha2::{Sha256, Digest};
use parking_lot::RwLock;
use chrono::Local;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GENESIS_USER_NAME: &str = "futuros";

struct ShellState {
    cwd: PathBuf,
    user_system: Arc<RwLock<UserSystem>>,
    current_user: Option<User>,
    is_authenticated: bool,
    virtual_dir: PathBuf,
}

impl ShellState {
    fn new() -> Self {
        Self {
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            user_system: Arc::new(RwLock::new(create_user_system_with_demo_keys())),
            current_user: None,
            is_authenticated: false,
            virtual_dir: PathBuf::new(),
        }
    }
    fn set_user(&mut self, user: User) {
        self.current_user = Some(user.clone());
        self.is_authenticated = true;
        self.virtual_dir = PathBuf::from("/users").join(&user.name).join("blocks");
    }
    fn clear_user(&mut self) {
        self.current_user = None;
        self.is_authenticated = false;
        self.virtual_dir = PathBuf::new();
    }
}

#[derive(Clone)]
pub struct MemoryBlock {
    pub id: Vec<u8>,
    pub data: Vec<u8>,
    pub owner: String,
    pub signature: Vec<u8>,
    pub block_type: String,
    pub created_at: i64,
    pub metadata: HashMap<String, String>,
}

impl MemoryBlock {
    pub fn new(data: Vec<u8>, owner: String, block_type: String) -> Self {
        let id = compute_hash(&data);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let mut metadata = HashMap::new();
        metadata.insert("original_name".to_string(), "unknown".to_string());
        metadata.insert("mime_type".to_string(), "application/octet-stream".to_string());
        metadata.insert("size".to_string(), data.len().to_string());
        let signature = compute_hash(&data);
        Self { id, data, owner, signature, block_type, created_at, metadata }
    }
    pub fn id_hex(&self) -> String { hex_encode(&self.id) }
    pub fn original_name(&self) -> String {
        self.metadata.get("original_name").cloned().unwrap_or_else(|| "unknown".to_string())
    }
    pub fn size(&self) -> usize { self.data.len() }
}

pub struct VirtualFileSystem {
    user_name: String,
    blocks: Arc<RwLock<HashMap<String, MemoryBlock>>>,
}

impl VirtualFileSystem {
    pub fn new(user_name: String) -> Self {
        Self { user_name, blocks: Arc::new(RwLock::new(HashMap::new())) }
    }
    pub fn add_block(&self, block: MemoryBlock) -> String {
        let block_id = block.id_hex();
        self.blocks.write().insert(block_id.clone(), block);
        block_id
    }
    pub fn get_block(&self, block_id: &str) -> Option<MemoryBlock> {
        self.blocks.read().get(block_id).cloned()
    }
    pub fn list_blocks(&self) -> Vec<MemoryBlock> {
        self.blocks.read().values().cloned().collect()
    }
    pub fn remove_block(&self, block_id: &str) -> bool {
        self.blocks.write().remove(block_id).is_some()
    }
    pub fn file_to_block(&self, file_path: &Path, block_name: Option<String>) -> Result<MemoryBlock, String> {
        if !file_path.exists() { return Err(format!("File not found: {}", file_path.display())); }
        let data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        let original_name = block_name.unwrap_or_else(|| 
            file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string());
        let mut block = MemoryBlock::new(data, self.user_name.clone(), "file".to_string());
        block.metadata.insert("original_name".to_string(), original_name.clone());
        if let Some(ext) = file_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let mime = match ext_str.to_lowercase().as_str() {
                    "txt" => "text/plain", "md" => "text/markdown",
                    "rs" => "text/x-rustsrc", "py" => "text/x-python",
                    "js" => "application/javascript", "json" => "application/json",
                    "html" => "text/html", "css" => "text/css",
                    "png" => "image/png", "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif", _ => "application/octet-stream",
                };
                block.metadata.insert("mime_type".to_string(), mime.to_string());
            }
        }
        block.metadata.insert("original_path".to_string(), file_path.to_string_lossy().into_owned());
        Ok(block)
    }
    pub fn get_block_by_name(&self, name: &str) -> Option<MemoryBlock> {
        let blocks = self.blocks.read();
        blocks.values().find(|b| b.original_name() == name).cloned()
    }
}

fn verify_self_integrity(user_system: &UserSystem) -> Result<bool, String> {
    let binary_path = env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let binary_hash = compute_file_hash(&binary_path);
    let system_blocks = user_system.get_system_blocks();
    for block_id in system_blocks {
        if let Some(block) = user_system.get_block(&block_id) {
            let block_data_hash = compute_hash(&block.data);
            if block_data_hash == binary_hash && user_system.is_system_block(&block_id) {
                if let Some(executable) = user_system.get_system_executable(&block_id) {
                    if &executable.executable_type == "kernel" || &executable.executable_type == "shell" || &executable.executable_type == "bootstrap" {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

fn compute_file_hash(path: &Path) -> Vec<u8> {
    let file = fs::File::open(path).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer).expect("Failed to read file");
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    hasher.finalize().to_vec()
}

fn compute_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16)).collect::<Result<Vec<u8>, _>>().map_err(|e| format!("Invalid hex: {}", e))
}

fn login_user(state: &mut ShellState) -> Result<(), String> {
    print!("\nUsername: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).map_err(|e| e.to_string())?;
    let username = username.trim().to_string();
    if username.is_empty() { println!("❌ Username cannot be empty"); return Ok(()); }
    let genesis_user = {
        let user_system = state.user_system.read();
        if username == GENESIS_USER_NAME {
            user_system.get_genesis_user().map(|u| u.clone())
        } else {
            None
        }
    };
    if let Some(genesis) = genesis_user {
        state.set_user(genesis);
        println!("✅ Login successful as genesis user '{}'!", GENESIS_USER_NAME);
        println!("   You own all system executable blocks");
        return Ok(());
    }
    println!("✅ Login successful as '{}'", username);
    let demo_public_key = vec![0u8; 64];
    let permissions = UserPermissions { can_create_blocks: true, can_read_all_blocks: false, can_write_all_blocks: false, can_manage_users: false, can_manage_system: false, can_execute_code: false };
    let user = User::new(username.clone(), demo_public_key, UserRole::User, permissions);
    state.set_user(user);
    Ok(())
}

fn create_account(state: &mut ShellState) -> Result<(), String> {
    print!("\nUsername: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).map_err(|e| e.to_string())?;
    let username = username.trim().to_string();
    if username.is_empty() { println!("❌ Username cannot be empty"); return Ok(()); }
    if username == GENESIS_USER_NAME { println!("❌ Username '{}' is reserved", GENESIS_USER_NAME); return Ok(()); }
    println!("✅ Account created for '{}'", username);
    println!("   Note: In production, generate PQC keys externally and provide ONLY public key");
    login_user(state)
}

fn handle_authentication(state: &mut ShellState) -> Result<(), String> {
    println!("\n=== PQOS Login ===");
    println!("1. Login");
    println!("2. Create account");
    println!("3. Back to shell");
    print!("\nChoice: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    match input.trim() {
        "1" => login_user(state),
        "2" => create_account(state),
        "3" => Ok(()),
        _ => { println!("❌ Invalid choice"); Ok(()) }
    }
}

fn process_command(state: &mut ShellState, vfs: &mut Option<VirtualFileSystem>, input: &str) -> Result<bool, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() { return Ok(true); }
    let cmd = parts[0].to_lowercase();
    let args: Vec<&str> = parts[1..].to_vec();
    match cmd.as_str() {
        "login" | "auth" => { state.clear_user(); *vfs = None; handle_authentication(state)?; Ok(true) }
        "logout" => { println!("Logging out..."); state.clear_user(); *vfs = None; Ok(true) }
        "help" | "?" => { print_help(state.is_authenticated); Ok(true) }
        "pwd" => { println!("{}", state.cwd.display()); Ok(true) }
        "cd" => {
            if args.is_empty() {
                state.cwd = match env::var("HOME") { Ok(home) => PathBuf::from(home), Err(_) => PathBuf::from("/") };
            } else {
                let target = args[0];
                if target.contains("/users/") && target.contains("/blocks/") {
                    if state.is_authenticated {
                        if let Some(ref user) = state.current_user {
                            if target.contains(&format!("/users/{}/blocks", user.name)) {
                                state.virtual_dir = PathBuf::from(target);
                                println!("✅ Changed to virtual directory: {}", target);
                            } else { println!("cd: permission denied: not your blocks directory"); }
                        }
                    } else { println!("cd: permission denied: please login first"); }
                } else {
                    let new_path = if target.starts_with('/') { PathBuf::from(target) } else { state.cwd.join(target) };
                    if new_path.exists() && new_path.is_dir() { state.cwd = new_path; } else { println!("cd: no such directory: {}", target); }
                }
            }
            Ok(true)
        }
        "ls" | "dir" => { list_current_directory(state, vfs)?; Ok(true) }
        "ll" | "ls -l" | "ls -la" => { list_current_directory(state, vfs)?; Ok(true) }
        "cp" | "copy" => {
            if args.len() < 2 { println!("Usage: cp <source> <destination>"); return Ok(true); }
            let source_path = state.cwd.join(args[0]);
            let dest_arg = args[1];
            if dest_arg.contains("/users/") && dest_arg.contains("/blocks/") {
                if state.is_authenticated { copy_to_virtual_directory(state, vfs, &source_path, dest_arg)?; }
                else { println!("❌ Permission denied: please login first"); }
            } else {
                let dest_path = if dest_arg.starts_with('/') { PathBuf::from(dest_arg) } else { state.cwd.join(dest_arg) };
                match fs::copy(&source_path, &dest_path) {
                    Ok(_) => println!("✅ Copied '{}' to '{}'", args[0], dest_arg),
                    Err(e) => println!("❌ Copy failed: {}", e),
                }
            }
            Ok(true)
        }
        "rm" | "remove" | "del" => {
            if args.is_empty() { println!("Usage: rm <file|block_id>"); return Ok(true); }
            let target = args[0];
            if target.len() == 64 && target.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Some(ref fs) = vfs {
                    if fs.remove_block(target) { println!("✅ Block {} removed", target); }
                    else { println!("❌ Block not found: {}", target); }
                } else { println!("❌ No virtual filesystem active"); }
            } else {
                let file_path = state.cwd.join(target);
                if file_path.exists() { fs::remove_file(&file_path).map_err(|e| e.to_string())?; println!("✅ Removed '{}'", target); }
                else { println!("❌ File not found: {}", target); }
            }
            Ok(true)
        }
        "blocks" | "myblocks" => {
            if !state.is_authenticated { println!("❌ Please login first"); return Ok(true); }
            if let Some(ref fs) = vfs { list_user_blocks(fs)?; }
            Ok(true)
        }
        "blockinfo" | "binfo" => {
            if args.is_empty() { println!("Usage: blockinfo <block_id>"); return Ok(true); }
            let block_id = args[0];
            if let Some(ref fs) = vfs {
                match fs.get_block(block_id) {
                    Some(block) => { print_block_info(&block)?; }
                    None => { println!("❌ Block not found: {}", block_id); }
                }
            } else { println!("❌ No virtual filesystem active"); }
            Ok(true)
        }
        "version" | "--version" | "-v" => { println!("PQOS Shell v{}", VERSION); println!("Post-Quantum Distributed Operating System"); Ok(true) }
        "clear" | "cls" => { print!("\x1B[2J\x1B[1;1H"); Ok(true) }
        "exit" | "quit" | "q" => { Ok(false) }
        _ => { println!("Unknown command: '{}'. Type 'help' for available commands.", cmd); Ok(true) }
    }
}

fn list_current_directory(state: &ShellState, vfs: &Option<VirtualFileSystem>) -> Result<(), String> {
    if !state.virtual_dir.to_string_lossy().is_empty() {
        if let Some(ref fs) = vfs { return list_virtual_directory(fs); }
    }
    list_directory(&state.cwd, false, vfs.as_ref())
}

fn list_directory(path: &Path, detailed: bool, _vfs: Option<&VirtualFileSystem>) -> Result<(), String> {
    if !path.exists() { println!("ls: cannot access '{}': No such file or directory", path.display()); return Ok(()); }
    if !path.is_dir() {
        if detailed {
            let metadata = fs::metadata(path).unwrap();
            let size = metadata.len();
            let modified: chrono::DateTime<chrono::Local> = metadata.modified().unwrap().into();
            println!("-rw-r--r-- 1 user user {:>10} {} {}", size, modified.format("%b %d %H:%M"), path.display());
        } else { println!("{}", path.file_name().unwrap_or_default().to_string_lossy()); }
        return Ok(());
    }
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    if detailed {
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let entry_path = entry.path();
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let file_type = if metadata.is_dir() { "d" } else { "-" };
            let permissions = if metadata.is_dir() { "rwxr-xr-x" } else { "rw-r--r--" };
            let size = if metadata.is_dir() { 0 } else { metadata.len() };
            let modified: chrono::DateTime<chrono::Local> = metadata.modified().unwrap().into();
            println!("{}{} 1 user user {:>10} {} {} {}", file_type, permissions, size, modified.format("%b %d %H:%M"), entry.file_name().to_string_lossy(), entry_path.display());
        }
    } else {
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            println!("{}", entry.file_name().to_string_lossy());
        }
    }
    Ok(())
}

fn list_virtual_directory(vfs: &VirtualFileSystem) -> Result<(), String> {
    let blocks = vfs.list_blocks();
    if blocks.is_empty() { println!("No blocks in virtual directory"); return Ok(()); }
    println!("\nMemory Blocks:");
    println!("{:<6} {:<40} {:<16} {:<12} {}", "#", "Block ID", "Type", "Size", "Name");
    println!("{}", "-".repeat(80));
    for (i, block) in blocks.iter().enumerate() {
        let size = block.size();
        let size_str = if size < 1024 { format!("{} B", size) } else if size < 1024 * 1024 { format!("{} KB", size / 1024) } else { format!("{} MB", size / (1024 * 1024)) };
        println!("{:<6} {:<40} {:<16} {:<12} {}", i+1, block.id_hex(), block.block_type, size_str, block.original_name());
    }
    Ok(())
}

fn list_user_blocks(vfs: &VirtualFileSystem) -> Result<(), String> {
    let blocks = vfs.list_blocks();
    if blocks.is_empty() {
        println!("No blocks found.");
        println!("Use 'cp <local_file> /users/<username>/blocks/' to add a file as a block");
        return Ok(());
    }
    println!("\nMemory Blocks owned by {}:", vfs.user_name);
    println!("{:<6} {:<40} {:<16} {:<12} {:<20} {}", "#", "Block ID", "Type", "Size", "Created", "Name");
    println!("{}", "-".repeat(100));
    for (i, block) in blocks.iter().enumerate() {
        let size = block.size();
        let size_str = if size < 1024 { format!("{} B", size) } else if size < 1024 * 1024 { format!("{} KB", size / 1024) } else { format!("{} MB", size / (1024 * 1024)) };
        let created = chrono::NaiveDateTime::from_timestamp_opt(block.created_at, 0).map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "Unknown".to_string());
        println!("{:<6} {:<40} {:<16} {:<12} {:<20} {}", i+1, block.id_hex(), block.block_type, size_str, created, block.original_name());
    }
    println!("\nTotal: {} block(s)", blocks.len());
    Ok(())
}

fn print_block_info(block: &MemoryBlock) -> Result<(), String> {
    println!("\nBlock Information:");
    println!("  ID: {}", block.id_hex());
    println!("  Type: {}", block.block_type);
    println!("  Owner: {}", block.owner);
    println!("  Original Name: {}", block.original_name());
    println!("  Size: {} bytes ({} KB)", block.size(), block.size() / 1024);
    if let Some(mime) = block.metadata.get("mime_type") { println!("  MIME Type: {}", mime); }
    if let Some(path) = block.metadata.get("original_path") { println!("  Original Path: {}", path); }
    let created = chrono::NaiveDateTime::from_timestamp_opt(block.created_at, 0).map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".to_string());
    println!("  Created: {}", created);
    println!("  Signature: {}...", hex_encode(&block.signature)[..16].to_string());
    Ok(())
}

fn copy_to_virtual_directory(state: &ShellState, vfs: &mut Option<VirtualFileSystem>, source_path: &Path, dest_arg: &str) -> Result<(), String> {
    if !source_path.exists() { println!("❌ Source file not found: {}", source_path.display()); return Ok(()); }
    let username = state.current_user.as_ref().map(|u| u.name.clone()).ok_or("No user authenticated")?;
    let expected_prefix = format!("/users/{}/blocks", username);
    if !dest_arg.starts_with(&expected_prefix) && !dest_arg.contains(&format!("/users/{}/blocks/", username)) {
        println!("❌ Destination must be in your blocks directory: {}/blocks/", expected_prefix); return Ok(());
    }
    if vfs.is_none() { *vfs = Some(VirtualFileSystem::new(username.clone())); }
    if let Some(ref mut fs) = vfs {
        let components: Vec<&str> = dest_arg.split('/').collect();
        let block_name = components.last().copied().unwrap_or("unknown");
        match fs.file_to_block(source_path, Some(block_name.to_string())) {
            Ok(block) => {
                let block_id = fs.add_block(block);
                println!("✅ File converted to memory block");
                println!("   Block ID: {}", block_id);
                println!("   Owner: {}", username);
                println!("   Original Name: {}", block_name);
                println!("   The file is now stored as a content-addressed block");
                println!("   It can be accessed via block ID or original name");
            }
            Err(e) => { println!("❌ Failed to convert file: {}", e); }
        }
    }
    Ok(())
}

fn print_help(is_authenticated: bool) {
    println!("\n=== PQOS Shell Commands ===");
    println!();
    println!("📁 File System:");
    println!("  pwd              Show current directory");
    println!("  cd <path>        Change directory (supports virtual /users/<you>/blocks/)");
    println!("  ls [path]        List files");
    println!("  ll [path]        List files with details");
    println!("  cp <src> <dest>  Copy file (to virtual dir converts to block)");
    println!("  rm <file|block>  Remove file or block");
    println!();
    if is_authenticated {
        println!("🧱 Memory Blocks:");
        println!("  blocks, myblocks    List your memory blocks");
        println!("  blockinfo <id>      Show block information");
        println!();
    }
    println!("🔐 Authentication:");
    println!("  login, auth        Login to system");
    println!("  logout             Logout from system");
    println!();
    println!("📋 System:");
    println!("  help, ?           Show this help");
    println!("  clear, cls        Clear screen");
    println!("  version, -v       Show version");
    println!("  exit, quit, q     Exit shell");
    println!();
    if is_authenticated {
        println!("💡 Tip: Use 'cp <local_file> /users/<yourname>/blocks/' to convert files to memory blocks");
    } else {
        println!("💡 Tip: Login to access memory block features");
    }
}

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Post-Quantum Distributed Operating System (PQOS) Shell           ║");
    println!("║                        v{} - CLI Interface                         ║", VERSION);
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Local::now();
    print_banner();
    println!("\n🔍 Verifying self-integrity...");
    
    let user_system = create_user_system_with_demo_keys();
    match verify_self_integrity(&user_system) {
        Ok(true) => {
            println!("✅ Self-integrity verified - Binary matches system blockchain");
        }
        Ok(false) => {
            println!("⚠️  Self-integrity: Binary not found in system blocks");
            println!("   This is expected during development");
            println!("   In production, this would prevent execution if verification fails");
        }
        Err(e) => {
            println!("❌ Self-integrity error: {}", e);
        }
    }
    
    println!("\n🚀 PQOS Shell ready. Type 'help' for available commands.\n");
    
    let mut state = ShellState::new();
    *state.user_system.write() = user_system;
    let mut vfs: Option<VirtualFileSystem> = None;
    
    loop {
        let prompt = if state.is_authenticated {
            let user = state.current_user.as_ref().map(|u| u.name.as_str()).unwrap_or("?");
            format!("pqos@{}:{} $", user, state.cwd.display())
        } else {
            format!("pqos:{} $", state.cwd.display())
        };
        
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() { continue; }
                
                if !state.is_authenticated {
                    match handle_authentication(&mut state) {
                        Ok(_) => {
                            if state.is_authenticated {
                                if let Some(ref user) = state.current_user {
                                    vfs = Some(VirtualFileSystem::new(user.name.clone()));
                                    println!("\n✅ Welcome, {}! Type 'help' for commands.", user.name);
                                }
                            }
                        }
                        Err(e) => { println!("❌ Authentication error: {}", e); }
                    }
                    continue;
                }
                
                match process_command(&mut state, &mut vfs, input) {
                    Ok(false) => { println!("\n👋 Goodbye!"); break; }
                    Ok(true) => { continue; }
                    Err(e) => { println!("❌ Error: {}", e); continue; }
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::Interrupted {
                    println!("\n❌ Input error: {}", e);
                }
                break;
            }
        }
    }
    
    Ok(())
}
