#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PostscriptOperator {
    Abs,

    /// returns the sum of num1 and num2. If both operands are integers and the
    /// result is within integer range, the result is an integer; otherwise, the
    /// result is a real number
    ///
    /// Examples
    ///    3 4 add ⇒ 7
    ///    9.9 1.1 add ⇒ 11.0
    ///
    /// num1 num2 `add` sum
    Add,

    /// returns the result of subtracting num2 from num1. If both operands are
    /// integers and the result is within integer range, the result is an
    /// integer; otherwise, the result is a real number.
    ///
    /// num1 num2 `sub` difference
    Sub,

    /// returns the product of num1 and num2. If both operands are integers and the
    /// result is within integer range, the result is an integer; otherwise, the
    /// result is a real number
    ///
    /// num1 num2 `mul` product
    Mul,

    /// divides num1 by num2, producing a result that is always a real number even if
    /// both operands are integers. Use idiv instead if the operands are integers
    /// and an integer result is desired
    ///
    /// num1 num2 `div` quotient
    Div,

    /// divides int1 by int2 and returns the integer part of the quotient, with any
    /// fractional part discarded. Both operands of idiv must be integers and the
    /// result is an integer
    ///
    /// int1 int2 `idiv` quotient
    Idiv,

    Dict,
    Begin,
    Dup,

    /// associates key with value in the current dictionary—the one on the top
    /// of the dictionary stack. If key is already present in the current
    /// dictionary, def simply replaces its value; otherwise, def creates a new
    /// entry for key and stores value with it
    ///
    /// key value `def` –
    Def,

    ReadOnly,
    ExecuteOnly,
    NoAccess,
    False,
    True,
    End,
    CurrentFile,
    EExec,
    Array,
    ArrayStart,
    ArrayEnd,
    ProcedureStart,
    ProcedureEnd,
    CurrentDict,
    String,
    Exch,
    ReadString,
    Pop,

    /// replaces a single element of the value of the first operand. If the first
    /// operand is an array or a string, put treats the second operand as an index
    /// and stores the third operand at the position identified by the index,
    /// counting from 0. index must be in the range 0 to n − 1, where n is the
    /// length of the array or string. If it is outside this range, a rangecheck error
    /// occurs
    ///
    /// If the first operand is a dictionary, put uses the second operand as a
    /// key and the third operand as a value, and stores this key-value pair into
    /// dict. If key is already present as a key in dict, put simply replaces its
    /// value by any; otherwise, put creates a new entry for key and associates
    /// any with it. In LanguageLevel 1, if dict is already full, a dictfull error
    /// occurs
    ///
    /// If the value of array or dict is in global VM and any is a composite
    /// object whose value is in local VM, an invalidaccess error occurs
    Put,

    /// returns true if there is an entry in the dictionary dict whose key is key;
    /// otherwise, it returns false. dict does not have to be on the dictionary stack
    ///
    /// dict key `known` bool
    Known,

    Not,
    Get,
    Exec,
    If,
    IfElse,
    Lt,
    Le,
    Index,
    DefineFont,
    Mark,
    CloseFile,

    /// returns the number of elements in the value of its operand if the operand is
    /// an array, a packed array, or a string. If the operand is a dictionary,
    /// length returns the current number of entries it contains (as opposed to
    /// its maximum capacity, which is returned by maxlength). If the operand is
    /// a name object, the length returned is the number of characters in the
    /// text string that defines it
    ///
    /// array `length` int
    /// packedarray `length` int
    /// dict `length` int
    /// string `length` int
    /// name `length` int
    Length,

    /// (convert to executable) makes the object on the top of the operand stack have
    /// the executable instead of the literal attribute
    ///
    /// any `cvx` any
    Cvx,

    /// returns the capacity of the dictionary dict—in other words, the maximum
    /// number of entries that dict can hold using the virtual memory currently
    /// allocated to it. In LanguageLevel 1, maxlength returns the length operand
    /// of the dict operator that created the dictionary; this is the
    /// dictionary’s maximum capacity (exceeding it causes a dictfull error). In
    /// a LanguageLevels 2 and 3, which permit a dictionary to grow beyond its
    /// initial capacity, maxlength returns its current capacity, a number at
    /// least as large as that returned by the length operator
    ///
    ///  dict `maxlength` int
    MaxLength,

    /// pops two objects from the operand stack and pushes true if they are equal,
    /// or false if not. The definition of equality depends on the types of the
    /// objects being compared. Simple objects are equal if their types and values
    /// are the same. Strings are equal if their lengths and individual elements
    /// are equal. Other composite objects (arrays and dictionaries) are equal
    /// only if they share the same value. Separate values are considered unequal,
    /// even if all the components of those values are the same.
    ///
    /// This operator performs some type conversions. Integers and real numbers
    /// can be compared freely: an integer and a real number representing the
    /// same mathematical value are considered equal by eq. Strings and names
    /// can likewise be compared freely: a name defined by some sequence of
    /// characters is equal to a string whose elements are the same sequence of
    /// characters.
    ///
    /// The literal/executable and access attributes of objects are not considered
    /// in comparisons between objects
    Eq,
    Ne,

    /// counts the number of items on the operand stack and pushes this count on
    /// the operand stack
    ///
    /// any1 … anyn count any1 … anyn n
    Count,

    /// returns a name object that identifies the type of the object any
    ///
    /// any `type` name
    Type,

    /// Register named resource instance in category
    ///
    /// key instance category `defineresource` instance
    DefineResource,

    /// Remove resource registration
    ///
    /// key category `undefineresource` –
    UndefineResource,

    /// Return resource instance identified by key in category
    ///
    /// key category `findresource` instance
    FindResource,

    /// Select CIE-based color rendering dictionary by rendering intent
    ///
    /// renderingintent `findcolorrendering` name bool
    FindColorRendering,

    /// Return status of resource instance
    ///
    /// key category `resourcestatus` status size true OR false
    ResourceStatus,

    /// Enumerate resource instances in category
    ///
    /// template proc scratch category `resourceforall` –
    ResourceForAll,

    /// executes the procedure proc repeatedly, passing it a sequence of values
    /// from initial by steps of increment to limit. The for operator expects
    /// initial, increment, and limit to be numbers. It maintains a temporary
    /// internal variable, known as the control variable, which it first sets to
    /// initial. Then, before each repetition, it compares the control variable
    /// to the termination value limit. If limit has not been exceeded, for pushes
    /// the control variable on the operand stack, executes proc, and adds increment
    /// to the control variable
    ///
    /// The termination condition depends on whether increment is positive or
    /// negative. If increment is positive, for terminates when the control variable
    /// becomes greater than limit. If increment is negative, for terminates when
    /// the control variable becomes less than limit. If initial meets the termination
    /// condition, for does not execute proc at all. If proc executes the exit
    /// operator, for terminates prematurely
    ///
    /// Usually, proc will use the value on the operand stack for some purpose.
    /// However, if proc does not remove the value, it will remain there. Successive
    /// executions of proc will cause successive values of the control variable
    /// to accumulate on the operand stack
    ///
    /// Examples
    ///   0 1 1 4 {add} for ⇒ 10
    ///   1 2 6 { } for ⇒ 1 3 5
    ///   3 −.5 1 { } for ⇒ 3.0 2.5 2.0 1.5 1.0
    ///
    /// In the first example above, the value of the control variable is added
    /// to whatever is on the stack, so 1, 2, 3, and 4 are added in turn to a
    /// running sum whose initial value is 0. The second example has an empty
    /// procedure, so the successive values of the control variable are left on
    /// the stack. The last example counts backward from 3 to 1 by halves, leaving
    /// the successive values on the stack
    ///
    /// Beware of using real numbers instead of integers for any of the first
    /// three operands. Most real numbers are not represented exactly. This can
    /// cause an error to accumulate in the value of the control variable, with
    /// possibly surprising results. In particular, if the difference between
    /// initial and limit is a multiple of increment, as in the last example,
    /// the control variable may not achieve the limit value.
    ///
    /// initial increment limit proc `for` –
    For,

    /// creates a snapshot of the current state of virtual memory (VM) and returns
    /// a save object representing that snapshot. The save object is composite
    /// and logically belongs to the local VM, regardless of the current VM
    /// allocation mode.
    ///
    /// Subsequently, the returned save object may be presented to restore to
    /// reset VM to this snapshot.
    ///
    /// save also saves the current graphics state by pushing a copy of it on the
    /// graphics state stack in a manner similar to gsave. This saved graphics
    /// state is restored by restore and grestoreall
    ///
    /// `save` save
    Save,

    /// resets virtual memory (VM) to the state represented by the supplied save
    /// object—in other words, the state at the time the corresponding save
    /// operator was executed.
    ///
    /// If the current execution context supports job encapsulation and if save
    /// represents the outermost saved VM state for this context, then objects
    /// in both local and global VM revert to their saved state. If the current
    /// context does not support job encapsulation or if save is not the outermost
    /// saved VM state for this context, then only objects in local VM revert
    /// to their saved state; objects in global VM are undisturbed.
    ///
    /// restore can reset VM to the state represented by any save object that is
    /// still valid, not necessarily the one produced by the most recent save.
    /// After restoring VM, restore invalidates its save operand along with
    /// any other save objects created more recently than that one. That is, a
    /// VM snapshot can be used only once; to restore the same environment
    /// repeatedly, it is necessary to do a new save each time
    ///
    /// restore does not alter the contents of the operand, dictionary, or
    /// execution stack, except to pop its save operand. If any of these stacks
    /// contains composite objects whose values reside in local VM and are newer
    /// than the snapshot being restored, an invalidrestore error occurs. This
    /// restriction applies to save objects and, in LanguageLevel 1, to name
    /// objects
    ///
    /// restore does alter the graphics state stack: it performs the equivalent
    /// of a grestoreall and then removes the graphics state created by save from
    /// the graphics state stack. restore also resets several per-context parameters
    /// to their state at the time of save. These include:
    ///  - Array packing mode (see setpacking)
    ///  - VM allocation mode (see setglobal)
    ///  -  Object output format (see setobjectformat)
    ///  -  All user interpreter parameters (see setuserparams)
    ///
    /// save `restore` –
    Restore,

    /// replaces executable operator names in proc by their values. For each element
    /// of proc that is an executable name, bind looks up the name in the context
    /// of the current dictionary stack as if by the load operator. If the name is
    /// found and its value is an operator object, bind replaces the name with the
    /// operator in proc. If the name is not found or its value is not an operator,
    /// bind does not make a change
    ///
    /// For each procedure object contained within proc, bind applies itself
    /// recursively to that procedure, makes the procedure read-only, and stores
    /// it back into proc. bind applies to both arrays and packed arrays, but
    /// it treats their access attributes differently. It will ignore a read-only
    /// array; that is, it will neither bind elements of the array nor examine
    /// nested procedures. On the other hand, bind will operate on a packed array
    /// (which always has read-only or even more restricted access), disregarding
    /// its access attribute. No error occurs in either case
    ///
    /// The effect of bind is that all operator names in proc and in procedures
    /// nested within proc to any depth become tightly bound to the operators
    /// themselves. During subsequent execution of proc, the interpreter
    /// encounters the operators themselves rather than their names.
    ///
    /// In LanguageLevel 3, if the user parameter IdiomRecognition is true, then
    /// after replacing executable names with operators, bind compares proc with
    /// every template procedure defined in instances of the IdiomSet resource
    /// category. If it finds a match, it returns the associated substitute
    /// procedure.
    ///
    /// proc `bind` proc
    Bind,

    /// performs two entirely different functions, depending on the type of the
    /// topmost operand
    ///
    /// In the first form, where the top element on the operand stack is a
    /// nonnegative integer n, copy pops n from the stack and duplicates the top
    /// n elements on the stack as shown above. This form of copy operates only
    /// on the objects themselves, not on the values of composite objects
    ///
    /// Examples
    ///     (a) (b) (c) 2 copy ⇒ (a) (b) (c) (b) (c)
    ///     (a) (b) (c) 0 copy ⇒ (a) (b) (c)
    ///
    /// In the other forms, copy copies all the elements of the first composite
    /// object into the second. The composite object operands must be of the same
    /// type, except that a packed array can be copied into an array (and only
    /// into an array—copy cannot copy into packed arrays, because they are
    /// read-only). This form of copy copies the value of a composite object.
    /// This is quite different from dup and other operators that copy only the
    /// objects themselves. However, copy performs only one level of copying. It
    /// does not apply recursively to elements that are themselves composite
    /// objects; instead, the values of those elements become shared.
    ///
    /// In the case of arrays or strings, the length of the second object must be at
    /// least as great as the first; copy returns the initial subarray or
    /// substring of the second operand into which the elements were copied. Any
    /// remaining elements of array2 or string2 are unaffected.
    ///
    /// In the case of dictionaries, LanguageLevel 1 requires that dict2 have a
    /// length (as returned by the length operator) of 0 and a maximum capacity
    /// (as returned by the maxlength operator) at least as great as the length
    /// of dict1. LanguageLevels 2 and 3 do not impose this restriction, since
    /// dictionaries can expand when necessary
    ///
    /// The literal/executable and access attributes of the result are normally the
    /// same as those of the second operand. However, in LanguageLevel 1 the
    /// access attribute of dict2 is copied from that of dict1
    ///
    /// If the value of the destination object is in global VM and any of the
    /// elements copied from the source object are composite objects whose values
    /// are in local VM, an invalidaccess error occurs
    ///
    /// Example
    ///     /a1 [1 2 3] def
    ///     a1 dup length array copy ⇒ [1 2 3]
    ///
    /// any1 … anyn n `copy` any1 … anyn any1 … anyn
    Copy,

    /// returns the logical conjunction of the operands if they are boolean. If the
    /// operands are integers, and returns the bitwise “and” of their binary
    /// representations
    ///
    /// bool1 bool2 `and` bool3
    /// int1 int2 `and` int3
    And,

    /// returns the logical disjunction of the operands if they are boolean. If the
    /// operands are integers, or returns the bitwise “inclusive or” of their
    /// binary representations
    ///
    /// bool1 bool2 `or` bool3
    /// int1 int2 `or` int3
    Or,

    /// returns the least integer value greater than or equal to num1. The type of
    /// the result is the same as the type of the operand
    ///
    /// num1 `ceiling` num2
    Ceiling,
    Floor,
    Round,

    /// pops two objects from the operand stack and pushes true if the first operand
    /// is greater than or equal to the second, or false otherwise. If both
    /// operands are numbers, ge compares their mathematical values. If both
    /// operands are strings, ge compares them element by element, treating the
    /// elements as integers in the range 0 to 255, to determine whether the
    /// first string is lexically greater than or equal to the second. If the
    /// operands are of other types or one is a string and the other is a number,
    /// a typecheck error occurs
    ///
    /// num1 num2 `ge` bool
    /// string1 string2 `ge` bool
    Ge,

    /// pops two objects from the operand stack and pushes true if the first operand
    /// is greater than the second, or false otherwise. If both operands are
    /// numbers, gt compares their mathematical values. If both operands are
    /// strings, gt compares them element by element, treating the elements as
    /// integers in the range 0 to 255, to determine whether the first string is
    /// lexically greater than the second. If the operands are of other types or
    /// one is a string and the other is a number, a typecheck error occurs
    ///
    /// num1 num2 `gt` bool
    /// string1 string2 `gt` bool
    Gt,

    /// pushes the internal dictionary object on the operand stack. The int operand
    /// must be the integer 1183615869. The internal dictionary is in local VM
    /// and is writeable. It contains operators and other information whose
    /// purpose is internal to the PostScript interpreter. It should be
    /// referenced only in special circumstances, such as during construction of
    /// Type 1 font programs. (See the book Adobe Type 1 Font Format for specific
    /// information about constructing Type 1 fonts.) The contents of
    /// internaldict are undocumented and subject to change at any time
    ///
    /// int `internaldict` dict
    InternalDict,
}
