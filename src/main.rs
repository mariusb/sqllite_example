// main.rs

// We need the rusqlite crate to interact with SQLite.
// Add it to your Cargo.toml with:
// cargo add rusqlite --features "bundled"
use rusqlite::{Connection, Result};

/// A declarative macro that defines a struct and implements a `create_table`
/// function for it. This function generates an SQLite table based on the
/// struct's definition.
macro_rules! sqlite_from_struct {
    (
        // Match attributes like `#[derive(Debug)]`
        $(#[$outer:meta])*
        // Match the struct keyword and its name (e.g., `struct User`)
        struct $struct_name:ident {
            // Match each field within the struct
            $(
                // Match attributes on fields, if any
                $(#[$inner:meta])*
                // Match the field name and its type (e.g., `id: i32`)
                $field_name:ident: $field_type:ty
            ),* // Match zero or more fields, separated by commas
            $(,)? // Allow an optional trailing comma
        }
    ) => {
        // --- Step 1: Re-create the original struct ---
        // The macro consumes the struct definition, so we must regenerate it
        // to make it available to the rest of the program.
        $(#[$outer])*
        struct $struct_name {
            $(
                $(#[$inner])*
                pub $field_name: $field_type,
            )*
        }

        // --- Step 2: Implement the `create_table` function for the struct ---
        impl $struct_name {
            /// Connects to an SQLite database and creates a table corresponding
            /// to the struct's schema.
            ///
            /// # Arguments
            ///
            /// * `conn` - A reference to an open SQLite connection.
            ///
            /// # Returns
            ///
            /// * `rusqlite::Result<()>` - An empty Ok result on success, or an Err on failure.
            pub fn create_table(conn: &Connection) -> Result<()> {
                // --- Step 3: Build the "CREATE TABLE" SQL string ---

                // Derive table name from struct name (e.g., User -> users)
                let table_name = stringify!($struct_name).to_lowercase() + "s";
                let mut create_sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);

                // Iterate over each field provided in the macro input
                $(
                    // Get the field name as a string
                    let column_name = stringify!($field_name);
                    // Get the field type as a string
                    let type_name = stringify!($field_type);

                    // Map Rust types to SQLite column types
                    let sql_type = match type_name {
                        "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => "INTEGER",
                        "f32" | "f64" => "REAL",
                        "String" | "&str" => "TEXT",
                        "bool" => "INTEGER", // SQLite uses 0 for false, 1 for true
                        "Vec<u8>" => "BLOB",
                        _ => "TEXT", // Default to TEXT for unknown types
                    };

                    // By convention, if a field is `id: i32`, make it the primary key.
                    if column_name == "id" && sql_type == "INTEGER" {
                        create_sql.push_str(&format!("    {} {} PRIMARY KEY AUTOINCREMENT,\n", column_name, sql_type));
                    } else {
                        create_sql.push_str(&format!("    {} {},\n", column_name, sql_type));
                    }
                )*

                // Remove the last comma and newline if the SQL string is not empty
                if create_sql.ends_with(",\n") {
                    create_sql.pop(); // remove \n
                    create_sql.pop(); // remove ,
                }

                // Close the SQL statement
                create_sql.push_str("\n);");

                // Print the generated SQL for verification
                println!("--- Generated SQL ---");
                println!("{}", create_sql);
                println!("---------------------");

                // --- Step 4: Execute the SQL statement ---
                conn.execute(&create_sql, [])?;

                println!("Successfully created table '{}'.", table_name);

                Ok(())
            }
        }
    };
}

// Use the macro to define a `User` struct.
// This will create the `User` struct AND the `User::create_table` function.
sqlite_from_struct! {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        id: i32,
        name: String,
        email: String,
        age: u32,
        is_active: bool,
    }
}

// Define another struct for demonstration purposes.
sqlite_from_struct! {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Product {
        id: i32,
        name: String,
        price: f64,
        in_stock: bool,
        image_data: Vec<u8>, // Will be mapped to BLOB
    }
}


fn main() {
    let db_path = "company.db";
    let conn = match Connection::open(db_path) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to open database '{}': {}", db_path, e);
            return;
        }
    };

    // --- Create the 'users' table ---
    match User::create_table(&conn) {
        Ok(_) => println!("User table creation successful.\n"),
        Err(e) => eprintln!("Error creating user table: {}\n", e),
    }

    // --- Create the 'products' table in the same database ---
    match Product::create_table(&conn) {
        Ok(_) => println!("Product table creation successful."),
        Err(e) => eprintln!("Error creating product table: {}", e),
    }
}
