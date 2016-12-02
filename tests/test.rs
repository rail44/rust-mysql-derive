#![feature(test, proc_macro)]

#[macro_use]
extern crate mysql_derive;
extern crate mysql;
extern crate test;

#[derive(FromMysqlRow, Debug, PartialEq, Eq)]
struct User {
    name: String,
    age: u64,
}

fn prepare_fixture(mut conn: mysql::PooledConn) {
    conn.query("CREATE DATABASE rust_mysql_derive").unwrap();
    conn.query("CREATE TABLE rust_mysql_derive.users (name varchar(32), age int)").unwrap();
    conn.query("INSERT INTO rust_mysql_derive.users VALUES ('alice', 20), ('bob', 21)").unwrap();
}

fn drop_fixture(mut conn: mysql::PooledConn) {
    conn.query("DROP DATABASE rust_mysql_derive").unwrap();
}

#[test]
fn test() {
    let pool = mysql::Pool::new("mysql://root:password@localhost:3306/").unwrap();
    prepare_fixture(pool.get_conn().unwrap());

    let users: Vec<User> = pool.prep_exec("SELECT * FROM rust_mysql_derive.users", ())
        .unwrap()
        .map(|opt_row| mysql::from_row(opt_row.unwrap()))
        .collect();

    drop_fixture(pool.get_conn().unwrap());

    assert_eq!(users,
               vec![User {
                        name: "alice".to_string(),
                        age: 20,
                    },
                    User {
                        name: "bob".to_string(),
                        age: 21,
                    }]);
}
