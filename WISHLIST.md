# SimplicityHL standard library

This document collects feature requests for the standard library. Some of these have already been implemented.

# General discussion

General question: to what extent should we wrap jets in order to discourage or obviate calling jets directly?

I've heard the suggestion that calling jets directly might eventually be rare in SimplicityHL (much as calling syscalls directly is rare in C, and one normally calls into libc for most purposes). But many existing jets do a specific task that a developer would specifically want to perform: many of them seem to exist at an appropriate and useful level of abstraction for SimplicityHL development purposes.

# Boolean operators

Maybe not needed if these are already rolled out as infix operators. I would somewhat prefer to have the compiler support these as `&&`, `||`, `!`.

* `and`
  * Implementation:
  * ```javascript
    fn and(a: bool, b: bool) -> bool {
        match a {
            true => b,
            false => false,
        }
    }
    ```
* `or`
  * Implementation:
  * ```javascript
    fn or(a: bool, b: bool) -> bool {
        match a {
            true => true,
            false => b,
        }
    }
    ```
* `not`
  * Implementation:
  * ```javascript
    fn not(a: bool) -> bool {
        match a {
            true => false,
            false => true,
        }
    }
    ```

# Asset and amount utilities

(Input/output by index.)

* Get asset
* Get asset amount
* Get asset and amount
* Check that the asset is equal to the passed value
* Check that the asset amount is equal to the passed value
* Check that the asset and amount are equal to the passed values
* Check that the input asset equals the output asset
* Check that the input asset amount equals the output asset amount
* Check that the input asset and amount equal the output asset and amount
* Same as prior three functions, but also check that the values are equal to the passed values

Q. Aren't the first three directly implemented by jets? Do we just need to `unwrap`?

# Script hash utilities

Input/output by index.

* Get script hash
* Check that the script hash is equal to the passed value
* Check that the input script hash equals the output script hash
* Same as prior function, but also check that the values are equal to the passed value

Q. Isn't the first one directly implemented by a jet? Do we just need to `unwrap` in case of a potentially-nonexistent input or output?

# `OP_RETURN`

* `ensure_op_return` function
* get `OP_RETURN` data

# Math

* Get decimals function (10^x)
* General integer division for all pairs of numeric types where the denominator's type is no larger than the numerator's type
* Checked arithmetic operations (automatically assert no overflow)

# Storage (state management)

√ Store and load single uninterpreted `u256` value
* Merkle tree tools (maybe also codegen for Merkle tree manipulation based on a separate schema?)

# Covenants (high-level)

* `ScriptAuth`
* `AssetAuth`
* Change calculations when performing partial withdrawals? (send all remaining funds back to the covenant)

Q. Are there (or could there be) standard protocols or conventions for (1) covenants that want to be the only inputs and outputs in a transaction, (2) covenants that can cooperate or coexist with other transaction inputs or outputs somehow? Andrew and Russell were having a fairly detailed discussion about this in late 2025 (e.g. "a copy of this covenant must be output 0, but the permitted value being withdrawn can be sent to any number of different output addresses").

# Other

* Comparing complex types for equality (at least all built-in type aliases or common jet return types)
  * I would be happier to have this be some kind of compiler notation (Andrew disfavors having it by the default interpretation of `==` but maybe there is another notation we can use), with automatic recursive destructuring of each side of the equality test.
  * If we don't call it `==`, Gemini suggests calling it `deep_eq` (implemented as a compiler built-in macro in order to be able to compare arbitrary types).
* `is_none`, `is_some` (as generic macros)
  * We currently have `is_none<T>` where the type must be specified explicitly, but the compiler could figure out what it is and not require specifying it.
* Oracle interpretation, once some oracle formats are standardized
* Relative timelocks (replacements for deprecated jets).
  * Implementations:
  * ```javascript
    fn enforce_relative_distance(min_distance: Distance) {
        // Assert that the current input is spent in a transaction that can
        // only appear a distance of at least min_distance blocks after the input's
        // UTXO. Panic otherwise.
    
        // Transaction version must be at least 2.
        assert!(jet::le_32(2, jet::version()));
    
        // Fetch and parse sequence for current transaction
        let parsed_seq: Option<Either<Distance, Duration>> = jet::parse_sequence(jet::current_sequence());
    
        match parsed_seq {
            // Failure condition
            None => assert!(false),
            // This is either a distance or a duration, but only a distance is
            // acceptable here.
            Some(actual_data: Either<Distance, Duration>) => match actual_data {
                // Is the actual distance greater than or equal to the specified min_distance?
                Left(actual_distance: Distance) => assert!(jet::le_16(min_distance, actual_distance)),
                // A duration is not acceptable in this context.
                Right(actual_duration: Duration) => assert!(false),
            },
        }
    }
    
    fn enforce_relative_duration(min_duration: Duration) {
        // Assert that the current input is spent in a transaction that can only
        // appear a duration of at least min_duration units of 512 seconds after
        // the input's UTXO. Panic otherwise.
    
        // Transaction version must be at least 2.
        assert!(jet::le_32(2, jet::version()));
    
        // Fetch and parse sequence for current transaction
        let parsed_seq: Option<Either<Distance, Duration>> = jet::parse_sequence(jet::current_sequence());
    
        match parsed_seq {
            // Failure condition
            None => assert!(false),
            // This is either a distance or a duration, but only a duration is
            // acceptable here.
            Some(actual_data: Either<Distance, Duration>) => match actual_data {
                // A distance is not acceptable in this context.
                Left(actual_distance: Distance) => assert!(false),
                // Is the actual duration greater than or equal to the specified min_duration?
                Right(actual_duration: Duration) => assert!(jet::le_16(min_duration, actual_duration)),
            },
        }
    }
    
    ```
    * See also <https://docs.simplicity-lang.org/documentation/timelock> 
* Fee management?
* More convenient SHA256?
