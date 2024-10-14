use comfy_table::Color::Green;
use comfy_table::{Cell, CellAlignment, Table};
use dirs::home_dir;
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use termion::clear;
use termion::cursor;

fn main() {
    let home_dir = home_dir().unwrap().display().to_string();
    let conn = Connection::open_with_flags(
        format!("{home_dir}/.config/sshm.db"),
        OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .expect("无法获取数据库连接!");

    let _ = conn.execute(
        "
        CREATE TABLE  IF NOT EXISTS servers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        host TEXT NOT NULL,
        port INTEGER DEFAULT 22,
        username TEXT NOT NULL,
        password BLOB
    );
        ",
        [],
    );
    // 匹配纯数字
    let num_regex = Regex::new(r"^\d+$").unwrap();
    loop {
        clear_screen();
        println!("                     欢迎使用SSHM管理工具                     ");
        list_servers(&conn);

        println!(
            "请输入服务器编号连接到服务器
或输入操作: c. 新增服务器 u. 更新服务器 d. 删除服务器  q. 退出"
        );
        let choice = read_input();
        match num_regex.is_match(choice.as_str()) {
            true => {
                connect_server(&conn, choice.parse().unwrap());
                break;
            }
            false => match choice.as_str() {
                "c" => {
                    add_server(&conn);
                }
                "u" => {
                    println!("请输入要更新的服务器 ID,输入非数字字符返回上层:");
                    let id = read_input();
                    match num_regex.is_match(id.as_str()) {
                        true => update_server(&conn, id.parse().unwrap()),
                        false => {
                            continue;
                        }
                    }
                }
                "d" => {
                    println!("请输入要删除的服务器 ID,输入非数字字符返回上层:");
                    let id = read_input();
                    match num_regex.is_match(id.as_str()) {
                        true => delete_server(&conn, id.parse().unwrap()),
                        false => {
                            continue;
                        }
                    }
                }
                "q" => break,
                _ => println!("无效的选择"),
            },
        }
    }
}

// 辅助函数，用于读取用户输入
fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("读取输入失败");
    input.trim().to_string()
}

/**
 * 主机列表
 */
fn list_servers(conn: &Connection) {
    let mut stmt = conn
        .prepare("SELECT id, name, host, port, username FROM servers")
        .expect("查询失败");
    let server_iter = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .expect("遍历数据失败");

    let mut table = Table::new();
    // table.set_header(vec!["服务器编号", "服务器别名", "服务器信息"]);
    table.set_header(vec![
        Cell::new("服务器编号")
            .set_alignment(CellAlignment::Center)
            .fg(Green),
        Cell::new("服务器别名")
            .set_alignment(CellAlignment::Center)
            .fg(Green),
        Cell::new("服务器信息")
            .set_alignment(CellAlignment::Center)
            .fg(Green),
    ]);
    for server in server_iter {
        let (id, name, host, port, username) = server.expect("读取数据失败");
        let server_info = format!("{username}@{host}:{port}");
        // table.add_row(vec![id.to_string(), name, server_info]);
        table.add_row(vec![
            Cell::new(id.to_string())
                .set_alignment(CellAlignment::Center)
                .fg(Green),
            Cell::new(name)
                .set_alignment(CellAlignment::Center)
                .fg(Green),
            Cell::new(server_info)
                .set_alignment(CellAlignment::Center)
                .fg(Green),
        ]);
    }
    println!("{table}")
}

/**
 * 添加主机
 */
fn add_server(conn: &Connection) {
    println!("请输入服务器名称:");
    let name = read_input();
    println!("请输入主机地址:");
    let host = read_input();
    println!("请输入端口号 (默认 22):");
    let port: i32 = read_input().parse().unwrap_or(22);
    println!("请输入用户名:");
    let username = read_input();
    println!("请输入密码:");
    let password = read_input();

    // let encrypted_password = encrypt_password(&password);
    conn.execute(
        "INSERT INTO servers (name, host, port, username, password) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, host, port, username, password],
    )
    .expect("插入数据失败");
}

/**
 * 更新目标主机
 */
fn update_server(conn: &Connection, server_id: i32) {
    let mut stmt = conn
        .prepare("SELECT id, name, host, port, username FROM servers where id =?1")
        .expect("查询失败");
    let server_iter = stmt
        .query_map(params![server_id], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .expect("遍历数据失败");

    for server in server_iter {
        let (id, name, host, port, username) = server.expect("读取数据失败");
        println!("请输入服务器名称: 原值=>{name}");
        let new_name = read_input();
        println!("请输入主机地址: 原值=>{host}");
        let new_host = read_input();
        println!("请输入端口号 (默认 22): 原值=>{port}");
        let new_port: i32 = read_input().parse().unwrap_or(22);
        println!("请输入用户名: 原值=>{username}");
        let new_username = read_input();
        println!("请输入密码:");
        let new_password = read_input();
        conn.execute(
            "update servers set name=?1,host=?2,port=?3,username=?4,password=?5 where id=?6",
            params![new_name, new_host, new_port, new_username, new_password, id],
        )
        .expect("更新数据失败");
    }
    // let encrypted_password = encrypt_password(&password);
}

/**
 * 删除目标主机
 */
fn delete_server(conn: &Connection, server_id: i32) {
    conn.execute("DELETE FROM servers WHERE id = ?1", params![server_id])
        .expect("删除数据失败");
}

/**
 * 连接目标主机
 */
fn connect_server(conn: &Connection, server_id: i32) {
    let mut stmt = conn
        .prepare("SELECT name, host, port, username,password FROM servers where id =?1")
        .expect("查询失败");
    let server_iter = stmt
        .query_map(params![server_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .expect("遍历数据失败");

    for server in server_iter {
        let (name, host, port, username, password) = server.expect("读取数据失败");
        // 调用系统 ssh 命令
        let status = Command::new("sshpass")
            .arg("-p")
            .arg(password) // 传递密码
            .arg("ssh")
            .arg(format!("{username}@{host}")) // 用户名@主机
            .arg("-p")
            .arg(port.to_string()) // 指定端口
            .stdin(Stdio::inherit()) // 继承标准输入
            .stdout(Stdio::inherit()) // 继承标准输出
            .stderr(Stdio::inherit()) // 继承标准错误输出
            .status() // 执行命令
            .expect("无法执行 ssh 命令");
        if !status.success() {
            eprintln!("别名=>{name} {username}@{host}:{port} 连接失败");
        }
    }
}

fn clear_screen() {
    // 移动光标到屏幕的左上角
    print!("{}", cursor::Goto(1, 1));
    // 清除屏幕
    print!("{}", clear::All);
    // 刷新输出缓冲区，确保清屏操作立即生效
    let _ = io::stdout().flush();
}
