#[macro_use]
extern crate mysql;

use mysql as my;

#[derive(Debug, PartialEq, Eq)]
struct Payment {
    customer_id: i32,
    amount: i32,
    account_name: Option<String>,
}


fn main() {
    // user:passworf@ip:port/database
    let pool = my::Pool::new("mysql://root:123456@localhost:3306/rust").unwrap();
    // Let's create payment table.
    // Unwrap just to make sure no error happened.
    pool.prep_exec(r"CREATE  TABLE IF NOT EXISTS payment (
                         customer_id int not null,
                         amount int not null,
                         account_name text
                     )", ()).unwrap();

    let payments = vec![
        Payment { customer_id: 1, amount: 2, account_name: None },
        Payment { customer_id: 3, amount: 4, account_name: Some("foo".into()) },
        Payment { customer_id: 5, amount: 6, account_name: None },
        Payment { customer_id: 7, amount: 8, account_name: None },
        Payment { customer_id: 9, amount: 10, account_name: Some("bar".into()) },
    ];

    for mut stmt in pool.prepare(r"INSERT INTO payment
                                       (customer_id, amount, account_name)
                                   VALUES
                                       (:customer_id, :amount, :account_name)").into_iter() {
        for p in payments.iter() {

            stmt.execute(params! {
                "customer_id" => p.customer_id,
                "amount" => p.amount,
                "account_name" => &p.account_name,
            }).unwrap();
        }
    }

    // Let's select payments from database
    let selected_payments: Vec<Payment> =
        pool.prep_exec("SELECT customer_id, amount, account_name from payment", ())
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    let (customer_id, amount, account_name) = my::from_row(row);
                    Payment {
                        customer_id: customer_id,
                        amount: amount,
                        account_name: account_name,
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap(); // Unwrap `Vec<Payment>`

    println!("{:?}",selected_payments);
    println!("Yay!");
}