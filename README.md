## Todo

* Do something with NOR in simplifying an expression like !(a or b)
* ~~Might be able to scope fixed vars by block; would need fixed to not just be a boolean
    but an option of a struct with the line numbers for the scope (which would need to be
    adjusted by the mips optimizer when lines are removed)~~ done?
* Add some kind of constants library for both MIPS and MYPS to use
(things like logic enumerations; "Horizontal: 20"). Going to need to think long
and hard about how to implement logic types and other game constants here.