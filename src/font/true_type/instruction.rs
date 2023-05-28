#[derive(Debug, Clone, Copy)]
pub enum TrueTypeInstruction {
    AdjustAngle,

    /// Replaces the number at the top of the stack with its absolute value
    AbsoluteValue,

    /// Adds the top two numbers on the stack
    Add,

    /// Aligns the two points whose numbers are the top two items on the stack along
    /// an axis orthogonal to the projection vector.
    ///
    /// Pops two point numbers, p2 and p1, from the stack and makes the distance
    /// between them zero by moving both points along the freedom vector to the
    /// average of their projections along the projection vector.
    AlignPoints,

    /// Aligns the points whose numbers are at the top of the stack with the point
    /// referenced by rp0.
    ///
    /// Pops point numbers, p1, p2, , ploopvalue, from the stack and aligns those
    /// points with the current position of rp0 by moving each point pi so that
    /// the projected distance from pi to rp0 is reduced to zero. The number of
    /// points aligned depends up the current setting the state variable loop.
    AlignToReferencePoint,

    /// Takes the logical and of the top two stack elements.
    ///
    /// Pops the top two elements, e2 and e1, from the stack and pushes the result
    /// of a logical and of the two elements onto the stack. Zero is pushed if
    /// either or both of the elements are FALSE (have the value zero). One is
    /// pushed if both elements are TRUE (have a non-zero value).
    And,

    /// Calls the function identified by the number of the top of the stack.
    ///
    /// Pops a function identifier number, f, from the stack and calls the
    /// function identified by f. The instructions contained in the function body
    /// will be executed. When execution of the function is complete, the
    /// instruction pointer will move to the next location in the instruction
    /// stream where execution of instructions will resume.
    Call,

    /// Takes the ceiling of the number at the top of the stack.
    ///
    /// Pops a number n from the stack and pushes n , the least integer value
    /// greater than or equal to n. Note that the ceiling of n, though an integer
    /// value, is expressed as 26.6 fixed point number.
    Ceiling,

    /// Copies the indexed stack element to the top of the stack.
    ///
    /// Pops a stack element number, k, from the stack and pushes a copy of the
    /// kth stack element on the top of the stack. Since it is a copy that is
    /// pushed, the kth element remains in its original position. This feature
    /// is the key difference between the CINDEX[ ] and MINDEX[ ] instructions.
    ///
    /// A zero or negative value for k is an error.
    CopyIndex,

    /// Clears all elements from the stack
    Clear,

    /// Pops an integer from the stack. In non-debugging versions of the interpreter,
    /// the execution of instructions will continue. In debugging versions, available
    /// to font developers, an implementation dependent debugger will be invoked.
    ///
    /// This instruction is only for debugging purposes and should not be a part
    /// of a finished font. Some implementations do not support this instruction.
    Debug,

    /// Creates an exception to one or more CVT values, each at a specified point
    /// size and by a specified amount.
    DeltaC1,
    DeltaC2,
    DeltaC3,

    DeltaP1,
    DeltaP2,
    DeltaP3,

    /// Pushes n, the number of elements currently in the stack, onto the stack
    Depth,

    /// Divides the number second from the top of the stack by the number at the
    /// top of the stack.
    ///
    /// Pops two 26.6 fixed point numbers, n1 and n2 off the stack and pushes
    /// onto the stack the quotient obtained by dividing n2 by n1. The division
    /// takes place in the following fashion, n1 is shifted left by six bits and
    /// then divided by 2.
    Div,

    /// Duplicates the top element on the stack.
    ///
    /// Pops an element, e, from the stack, duplicates that element and pushes
    /// it twice.
    Dup,

    /// Marks the end of an IF or IF-ELSE instruction sequence
    EndIf,

    /// Marks the start of the sequence of instructions that are to be executed
    /// when an IF instruction encounters a FALSE value on the stack. This sequence
    /// of instructions is terminated with an EIF instruction.
    ///
    /// The ELSE portion of an IF-ELSE-EIF sequence is optional.
    Else,

    /// Marks the end of a function definition or an instruction definition.
    /// Function definitions and instruction definitions cannot be nested.
    EndFunctionDefinition,

    /// Tests whether the top two numbers on the stack are equal in value.
    ///
    /// Pops two 32 bit values, e2 and e1, from the stack and compares them. If
    /// they are the same, one, signifying TRUE is pushed onto the stack. If they
    /// are not equal, zero, signifying FALSE is placed onto the stack.
    Equal,

    /// Tests whether the number at the top of the stack, when rounded according
    /// to the round state, is even.
    ///
    /// Pops a 26.6 number, e, from the stack and rounds that number according
    /// to the current round state. The number is then truncated to an integer.
    /// If the truncated number is even, one, signifying TRUE, is pushed onto
    /// the stack; if it is odd, zero, signifying FALSE, is placed onto the stack.
    Even,

    /// Marks the start of a function definition and pops a number, f, from the
    /// stack to uniquely identify this function. That definition will terminate
    /// when an ENDF[] is encountered in the instruction stream. A function
    /// definition can appear only in the font program or the CVT program. Functions
    /// must be defined before they can be used with a CALL[ ] instruction.
    BeginFunctionDefinition,

    /// Sets the auto flip Boolean in the graphics state to FALSE causing the
    /// MIRP[] and MIAP[] instructions to use the sign of control value table
    /// entries. When auto flip is set to FALSE, the direction in which distances
    /// are measured becomes significant. The default value for the auto flip
    /// state variable is TRUE.
    FlipOff,

    /// Sets the auto flip Boolean in the graphics state to TRUE causing the MIRP[]
    /// and MIAP[] instructions to ignore the sign of control value table entries.
    /// When the auto flip variable is TRUE, the direction in which distances are
    /// measured becomes insignificant. The default value for the auto flip state
    /// variable is TRUE.
    FlipOn,

    /// Makes an on-curve point an off-curve point or an off-curve point an on-curve
    /// point.
    ///
    /// Pops points, p, p1, p2, , ploopvalue from the stack. If pi is an on-curve
    /// point it is made an off-curve point. If pi is an off-curve point it is
    /// made an on-curve point. None of the points pi is marked as touched. As a
    /// result, none of the flipped points will be affected by an IUP[ ] instruction.
    /// A FLIPPT[ ] instruction redefines the shape of a glyph outline.
    FlipPoint,

    /// Changes all of the points in the range specified to off-curve points.
    ///
    /// Pops two numbers defining a range of points, the first a highpoint and
    /// the second a lowpoint. On-curve points in this range will become off-curve
    /// points. The position of the points is not affected and accordingly the
    /// points are not marked as touched.
    FlipRangeOff,

    /// Makes all the points in a specified range into on-curve points.
    ///
    /// Pops two numbers defining a range of points, the first a highpoint and
    /// the second a lowpoint. Off-curve points in this range will become on-curve
    /// points. The position of the points is not affected and accordingly the
    /// points are not marked as touched.
    FlipRangeOn,

    /// Takes the floor of the value at the top of the stack.
    ///
    /// Pops a 26.6 fixed point number n from the stack and returns n , the greatest
    /// integer value less than or equal to n. Note that the floor of n, though
    /// an integer value, is expressed as 26.6 fixed point number.
    Floor,

    /// Gets the coordinate value of the specified point using the current
    /// projection vector.
    ///
    /// Pops a point number p and pushes the coordinate value of that point on
    /// the current projection vector onto the stack. The value returned by GC[]
    /// is dependent upon the current direction of the projection vector.
    GetProjectionVectorCoordinate,

    /// Used to obtain data about the version of the TrueType engine that is
    /// rendering the font as well as the characteristics of the current glyph.
    /// The instruction pops a selector used to determine the type of information
    /// desired and pushes a result onto the stack.
    ///
    /// Setting bit 0 in the selector requests the engine version. Setting bit
    /// 1 asks whether the glyph has been rotated. Setting bit 2 asks whether
    /// the glyph has been stretched. To request information on two or more of
    /// these values, set the appropriate bits. For example, a selector value
    /// of 6 (0112) requests information on both rotation and stretching.
    ///
    /// The result is pushed onto the stack with the requested information. Bits
    /// 0 through 7 of result comprise the font engine version number. The version
    /// numbers are listed in TABLE 0-2.
    ///
    /// Bit 8 is set to 1 if the current glyph has been rotated. It is 0 otherwise.
    /// Bit 9 is set to 1 to indicate that the glyph has been stretched. It is
    /// 0 otherwise.
    GetInfo,

    /// Decomposes the current freedom vector into its x and y components and puts
    /// those components on the stack as two 2.14 numbers. The numbers occupy the
    /// least significant two bytes of each long.
    ///
    /// The first component pushed, px, is the x-component of the freedom vector.
    /// The second pushed, py, is the y-component of the freedom vector. Each is
    /// a 2.14 number.
    ///
    /// GFV[] treats the freedom vector as a unit vector originating at the grid origin
    GetFreedomVector,

    /// Decomposes the current projection vector into its x and y components and
    /// pushes those components onto the stack as two 2.14 numbers.
    ///
    /// The first component pushed, px, is the x-component of the projection vector.
    /// The second pushed, py, is the y-component of the projection vector
    ///
    /// GPV[] treats the projection vector as a unit vector originating at the
    /// grid origin
    GetProjectionVector,

    /// Compares the size of the top two stack elements.
    ///
    /// Pops two integers, e2 and e1, from the stack and compares them. If e1 is
    /// greater than e2, one, signifying TRUE, is pushed onto the stack. If e1
    /// is not greater than e1, zero, signifying FALSE, is placed onto the stack.
    GreaterThan,

    /// Compares the size of the top two stack elements.
    ///
    /// Pops two integers, e2 and e1, from the stack and compares them. If e1 is
    /// greater than or equal to e2, one, signifying TRUE, is pushed onto the stack.
    /// If e1 is not greater than or equal to e1, zero, signifying FALSE, is
    /// placed onto the stack.
    GreaterThanOrEqual,

    /// Begins the definition of an instruction. The instruction is identified
    /// by the opcode popped. The intent of the IDEF[ ] instruction is to allow
    /// old versions of the scaler to work with fonts that use instructions defined
    /// in later releases of the TrueType interpreter. Referencing an undefined
    /// opcode will have no effect. The IDEF[ ] is not intended for creating user
    /// defined instructions. The FDEF[ ] should be used for that purpose.
    ///
    /// The instruction definition that began with the IDEF[ ] terminates when
    /// an ENDF[ ] is encountered in the instruction stream. Nested IDEFs are not
    /// allowed. Subsequent executions of the opcode popped will be directed to
    /// the contents of this instruction definition. IDEFs should be defined in
    /// the font program. Defining instructions in the CVT program is not recommended.
    InstructionDefinition,

    /// Marks the beginning of an if-statement.
    ///
    /// Pops an integer, e, from the stack. If e is zero (FALSE), the instruction
    /// pointer is moved to the associated ELSE or EIF[] instruction in the
    /// instruction stream. If e is nonzero (TRUE), the next instruction in the
    /// instruction stream is executed. Execution continues until the associated
    /// ELSE[] instruction is encountered or the associated EIF[] instruction
    /// ends the IF[] statement. If an associated ELSE[] statement is found before
    /// the EIF[], the instruction pointer is moved to the EIF[] statement.
    If,

    /// Sets the instruction control state variable making it possible to turn
    /// on or off the execution of instructions and to regulate use of parameters
    /// set in the CVT program.
    ///
    /// This instruction clears and sets various control flags. The selector is
    /// used to choose the relevant flag. The value determines the new setting
    /// of that flag.
    InstructionControl,

    /// Interpolates the position of the specified points to preserve their original
    /// relationship with the reference points rp1 and rp2.
    InterpolatePoint,

    /// Moves the specified point to the intersection of the two lines specified.
    IntersectLines,

    /// Interpolates untouched points in the zone referenced by zp2 to preserve
    /// the original relationship of the untouched points to the other points in
    /// that zone.
    InterpolateUntouchedPoints(u8),

    /// Moves the instruction pointer to a new location specified by the offset
    /// popped from the stack.
    ///
    /// Pops an integer offset from the stack. The signed offset is added to the
    /// instruction pointer and execution is resumed at the new location in the
    /// instruction steam. The jump is relative to the position of the instruction
    /// itself. That is, an offset of +1 causes the instruction immediately following
    /// the JMPR[] instruction to be executed.
    JumpRelative,

    /// Moves the instruction pointer to a new location specified by the offset
    /// popped from the stack if the element tested has a FALSE (zero) value.
    ///
    /// Pops a Boolean value, e and an offset. In the case where the Boolean, e,
    /// is FALSE, the signed offset will be added to the instruction pointer and
    /// execution will be resumed at the new location; otherwise, the jump is not
    /// taken. The jump is relative to the position of the instruction itself.
    JumpRelativeOnFalse,

    /// Moves the instruction pointer to a new location specified by the offset
    /// value popped from the stack if the element tested has a TRUE value.
    ///
    /// Pops a Boolean value, e and an offset. If the Boolean is TRUE (non-zero)
    /// the signed offset will be added to the instruction pointer and execution
    /// will be resumed at the address obtained. Otherwise, the jump is not taken.
    /// The jump is relative to the position of the instruction itself.
    JumpRelativeOnTrue,

    /// Repeatedly calls a function.
    ///
    /// Pops a function number f and a count. Calls the function, f, count number
    /// of times
    LoopAndCall,

    /// Compares the two number at the top of the stack. The test succeeds if the
    /// second of the two numbers is smaller than the first.
    ///
    /// Pops two integers from the stack, e2 and e1, and compares them. If e1
    /// is less than e2, 1, signifying TRUE, is pushed onto the stack. If e1 is
    /// not less than e2, 0, signifying FALSE, is placed onto the stack.
    LessThan,

    /// Compares the two numbers at the top of the stack. The test succeeds if
    /// the second of the two numbers is smaller than or equal to the first.
    ///
    /// Pops two integers, e2 and e1 from the stack and compares them. If e1 is
    /// less than or equal to e2, one, signifying TRUE, is pushed onto the stack.
    /// If e1 is greater than e2, zero, signifying FALSE, is placed onto the stack.
    LessThanOrEqual,

    /// Returns the larger of the top two stack elements.
    ///
    /// Pops two elements, e2 and e1, from the stack and pushes the larger of these
    /// two quantities onto the stack
    Max,

    /// Measures the distance between the two points specified.
    MeasureDistance,

    /// Touch and, in some cases, round the specified point. A point that is
    /// "dapped" will be unaffected by subsequent IUP[ ] instructions and is
    /// generally intended to serve as a reference point for future instructions.
    /// Dapping a point with rounding set to grid will cause the point to have
    /// an integer valued coordinate along the projection vector. If the projection
    /// vector is set to the x-axis or y-axis, this will cause the point to be
    /// grid-aligned.
    ///
    /// Pops a point number, p, and sets reference points rp0 and rp1 to point
    /// p. If the Boolean a is set to 1, the coordinate of point p, as measured
    /// against the projection vector, will be rounded and then moved the rounded
    /// distance from its current position. If the Boolean a is set to 0, point
    /// p is not moved, but nonetheless is marked as touched in the direction(s)
    /// specified by the current freedom vector.
    MoveDirectAbsolutePoint(u8),

    MoveDirectRelativePoint(u8),

    MoveIndirectAbsolutePoint(u8),

    /// Returns the minimum of the top two stack elements
    ///
    /// Pops two elements, e2 and e1, from the stack and pushes the smaller of
    /// these two quantities onto the stack
    Min,

    /// Moves the indexed element to the top of the stack thereby removing it
    /// from its original position
    ///
    /// Pops an integer, k, from the stack and moves the element with index k to
    /// the top of the stack
    MoveIndexed,

    MoveIndirectRelativePoint(u8),

    /// Pushes the current number of pixels per em onto the stack. Pixels per em
    /// is a function of the resolution of the rendering device and the current
    /// point size and the current transformation matrix. This instruction looks
    /// at the projection vector and returns the number of pixels per em in that
    /// direction. The number is always an integer.
    MeasurePixelsPerEm,

    /// Pushes the current point size onto the stack.
    ///
    /// Measure point size can be used to obtain a value which serves as the basis
    /// for choosing whether to branch to an alternative path through the instruction
    /// stream. It makes it possible to treat point sizes below or above a certain
    /// threshold differently.
    MeasurePointSize,

    /// Makes it possible to coordinate the distance between a point and a reference
    /// point by setting the distance from a value popped from the stack.
    ///
    /// Pops a distance, d and a point number, p, and makes the distance between
    /// point p and the current position of rp0 equal to d. The distance, d, is
    /// in pixel coordinates.
    ///
    /// MSIRP[ ] is very similar to the MIRP[ ] instruction except for taking the
    /// distance from the stack rather than the CVT. Since MSIRP[ ] does not use
    /// the CVT, the control value cut-in is not a factor as it is in MIRP[ ].
    /// Since MSIRP[ ] does not round, its effect is not dependent upon the round
    /// state.
    ///
    /// MSIRP[] can be used to create points in the twilight zone.
    MoveStackIndirectRelativePoint,

    /// Multiplies the top two numbers on the stack. Pops two 26.6 numbers, n2
    /// and n1, from the stack and pushes onto the stack the product of the two
    /// elements. The 52.12 result is shifted right by 6 bits and the high 26
    /// bits are discarded yielding a 26.6 result.
    Multiply,

    /// Negates the number at the top of the stack.
    ///
    /// Pops a number, n, from the stack and pushes the negated value of n onto
    /// the stack
    Negate,

    /// Determines whether the two elements at the top of the stack are unequal.
    ///
    /// Pops two numbers, e2 and e1, from the stack and compares them. If they
    /// are different, one, signifying TRUE is pushed onto the stack. If they
    /// are equal, zero, signifying FALSE is pushed onto the stack.
    NotEqual,

    /// Takes the logical negation of the number at the top of the stack
    ///
    /// Pops a number e from the stack and returns the result of a logical NOT
    /// operation performed on e. If e was zero, one is pushed onto the stack
    /// if e was nonzero, zero is pushed onto the stack
    LogicalNot,

    /// Takes n bytes from the instruction stream and pushes them onto the stack
    ///
    /// Looks at the next byte in the instructions stream, n, and takes n unsigned
    /// bytes from the instruction stream, where n is an unsigned integer in the
    /// range (0 255), and pushes them onto the stack. The number of bytes to push,
    /// n, is not pushed onto the stack
    ///
    /// Each byte value is unsigned extended to 32 bits before being pushed onto
    /// the stack
    PushNBytes,

    /// Takes n words from the instruction stream and pushes them onto the stack
    ///
    /// Looks at the next instruction stream byte n and takes n 16-bit signed
    /// words from the instruction stream, where n is an unsigned integer in the
    /// range (0 255), and pushes them onto the stack. Each word is sign extended
    /// to 32 bits before being placed on the stack.The value n is not pushed onto the stack
    PushNWords,

    /// Changes the values of the number at the top of the stack to compensate
    /// for the engine characteristics.
    ///
    /// Pops a value, n1, from the stack and, possibly, increases or decreases
    /// its value to compensate for the engine characteristics established with
    /// the Boolean setting ab. The result, n2, is pushed onto the stack.
    NoRound,

    /// Tests whether the number at the top of the stack is odd
    ///
    /// Pops a number, e1, from the stack and rounds it according to the current
    /// setting of the round state before testing it. The number is then truncated
    /// to an integer. If the truncated number is odd, one, signifying TRUE, is
    /// pushed onto the stack if it is even, zero, signifying FALSE is placed
    /// onto the stack
    Odd,

    /// Takes the logical or of the two numbers at the top of the stack
    ///
    /// Pops two numbers, e2 and e1 off the stack and pushes onto the stack the
    /// result of a logical or operation between the two elements. Zero is pushed
    /// if both of the elements are FALSE (have the value zero). One is pushed if
    /// either both of the elements are TRUE (has a nonzero value)
    LogicalOr,

    /// Pops the top element from the stack
    Pop,

    /// Takes the specified number of bytes from the instruction stream and pushes
    /// them onto the interpreter stack
    ///
    /// The variables a, b, and c are binary digits representing numbers from 000
    /// to 111 (0-7 in binary). The value 1 is automatically added to the abc
    /// figure to obtain the actual number of bytes pushed
    ///
    /// When byte values are pushed onto the stack they are non-sign extended
    /// with zeroes to form 32 bit numbers
    PushBytes(u8),

    /// Takes the specified number of words from the instruction stream and pushes
    /// them onto the interpreter stack
    ///
    /// The variables a, b, and c are binary digits representing numbers from
    /// 000 to 111 (0-7 binary). The value 1 is automatically added to the abc
    /// figure to obtain the actual number of bytes pushed
    ///
    /// When word values are pushed onto the stack they are sign extended to 32
    /// bits
    PushWords,

    /// Read a control value table entry and places its value onto the stack
    ///
    /// Pops a CVT location from the stack and pushes the value found in the
    /// location specified onto the stack
    ReadControlValueTableEntry,

    /// Sets the round state variable to down to grid. In this state, distances
    /// are first subjected to compensation for the engine characteristics and
    /// then truncated to an integer. If the result of the compensation and
    /// rounding would be to change the sign of the distance, the distance is
    /// set to 0
    RoundDownToGrid,

    /// Sets the round state variable to round off. In this state engine compensation
    /// occurs but no rounding takes place. If engine compensation would change
    /// the sign of a distance, the distance is set to 0
    RoundOff,

    /// Performs a circular shift of the top three stack elements
    ///
    /// Pops the top three stack elements, a, b, and c and performs a circular
    /// shift of these top three objects on the stack with the effect being to
    /// move the third element to the top of the stack and to move the first two
    /// elements down one position. ROLL is equivalent to MINDEX[] with the value
    /// 3 at the top of the stack
    Roll,

    /// Uses round state, freedom vector
    ///
    /// Rounds the value at the top of the stack while compensating for the engine
    /// characteristics
    ///
    /// Pops a 26.6 fixed point number, n1, and, depending on the engine
    /// characteristics established by Booleans ab, the result is increased or
    /// decreased by a set amount. The number obtained is then rounded according
    /// to the current rounding state and pushed back onto the stack as n2
    Round,

    /// Reads the value in the specified storage area location and pushes that
    /// value onto the stack
    ///
    /// Pops a storage area location, n, from the stack and reads a 32-bit value,
    /// v, from that location. The value read is pushed onto the stack. The number
    /// of available storage locations is specified in the 'maxp' table in the
    /// font file'
    ReadStore,

    /// Sets the round state variable to double grid. In this state, distances
    /// are compensated for engine characteristics and then rounded to an integer
    /// or half-integer, whichever is closest
    RoundToDoubleGrid,

    /// Sets the round state variable to grid. In this state, distances are
    /// compensated for engine characteristics and rounded to the nearest integer
    RoundToGrid,

    /// Sets the round state variable to half grid. In this state, distances are
    /// compensated for engine characteristics and rounded to the nearest half
    /// integer. If these operations change the sign of the distance, the distance
    /// is set to +1/2 or -1/2 according to the original sign of the distance
    RoundToHalfGrid,

    /// Sets the round state variable to up to grid. In this state, after compensation
    /// for the engine characteristics, distances are rounded up to the closest
    /// integer. If the compensation and rounding would change the sign of the
    /// distance, the distance will be set to 0
    RoundUpToGrid,

    /// S45ROUND[ ] is analogous to SROUND[ ]. The differ is that it uses a gridPeriod
    /// of pixels rather than 1 pixel. S45ROUND[ ] is useful for finely controlling
    /// rounding of distances that will be measured at a 45 angle to the x-axis
    SuperRound45Deg,

    /// Pops a 32 bit integer, weight, from the stack and sets the value of the
    /// angle weight state variable accordingly. This instruction is anachronistic.
    /// Except for popping a single stack element, it has no effect
    SetAngleWeight,

    /// Pops a number, n, which is decomposed to a set of flags specifying the
    /// dropout control mode. SCANCTRL is used to set the value of the graphics
    /// state variable scan control which in turn determines whether the scan
    /// converter will activate dropout control for this glyph
    ScanConversionControl,

    /// Used to choose between dropout control with subs and without stubs.
    ///
    /// Pops a stack element consisting of a16-bit integer extended to 32 bits.
    /// The value of this integer is used to determine which rules the scan
    /// converter will use. If the value of the argument is 2, the non-dropout
    /// control scan converter will be used. If the value of the integer is 0 or
    /// 1, the dropout control mode will be set
    ScanType,

    /// Moves a point to the position specified by the coordinate value given
    /// on the stack.
    ///
    /// Pops a coordinate value, c, and a point number, p, and moves point p from
    /// its current position along the freedom vector so that its component along
    /// the projection vector becomes the value popped off the stack.
    SetsCoordinateFromStack,

    /// Establish a new value for the control value table cut-in
    ///
    /// Pops a value, n, from the stack and sets the control value cut-in to n.
    /// Increasing the value of the cut-in will increase the range of sizes for
    /// which CVT values will be used instead of the original outline value
    SetControlValueTableCutIn,

    /// Establishes a new value for the delta base state variable thereby changing
    /// the range of values over which a DELTA[] instruction will have an affect
    ///
    /// Pops a number, n, and sets delta base to the value n. The default for
    /// delta base is 9
    SetDeltaBase,

    /// Sets a second projection vector based upon the original position of two
    /// points. The new vector will point in a direction that is parallel to the
    /// line defined from p2 to p1. The projection vector is also set in in a
    /// direction that is parallel to the line from p2 to p1 but it is set using
    /// the current position of those points
    ///
    /// Pops two point numbers from the stack and uses them to specify a line
    /// that defines a second, dual projection vector. This dual projection vector
    /// uses coordinates from the original outline before any instructions are
    /// executed. It is used only with the IP[], GC[], MD[], MDRP[] and MIRP[]
    /// instructions. The dual projection vector is used in place of the projection
    /// vector in these instructions. This continues until some instruction sets
    /// the projection vector again
    SetDualProjectionVector,

    /// Establish a new value for the delta shift state variable thereby changing
    /// the step size of the DELTA[] instructions
    ///
    /// Pops a value n from the stack and sets delta shift to n. The default for
    /// delta shift is 3
    SetDeltaShift,

    /// Changes the direction of the freedom vector using values take from the
    /// stack and thereby changing the direction in which points can move
    ///
    /// Sets the direction of the freedom vector using the values x and y taken
    /// from the stack. The vector is set so that its projections onto the x and
    /// y -axes are x and y, which are specified as signed (two's complement)
    /// fixed-point (2.14) numbers. The value (x2 + y2) must be equal to 1 (0x4000)
    SetFreedomVectorFromStack,

    SetFreedomVectorToCoordinateAxis,
    SetFreedomVectorToLine,

    /// Sets the freedom vector to be the same as the projection vector. This
    /// means that movement and measurement will be in the same direction
    SetFreedomVectorToProjectionVector,

    /// Shifts a contour by the amount that the reference point was shifted
    ShiftContourUsingReferencePoint,

    /// Shifts points specified by the amount the reference point has already been
    /// shifted
    ShiftPointUsingReferencePoint(u8),

    /// Shift the specified points by the specified amount
    ShiftPointByPixels,

    /// Shifts all of the points in the specified zone by the amount that the
    /// reference point has been shifted
    ShiftZoneUsingReferencePoint,

    /// Changes the value of the loop variable thereby changing the number of
    /// times the affected instructions will execute if called
    SetLoop,

    /// Establishes a new value for the minimum distance, the smallest possible
    /// value to which distances will be rounded. An appropriate setting for this
    /// variable can prevent distances from rounding to zero and therefore
    /// disappearing when grid-fitting takes place
    SetMinimumDistance,

    /// Establishes a new value for the projection vector using values taken from
    /// the stack
    SetProjectionVectorFromStack,

    /// Sets the projection vector to one of the coordinate axes depending on the
    /// value of the flag a
    SetProjectionVectorToCoordinateAxis,

    /// Changes the direction of the projection vector to that specified by the
    /// line defined by the endpoints taken from the stack. The order in which
    /// the points are specified is significant Reversing the order of the points
    /// will reverse the direction of the projection vector
    ///
    /// Pops two point numbers, p2 and p1 and sets the projection vector to a unit
    /// vector parallel or perpendicular to the line segment from point p2 to
    /// point p1 and pointing from p2 to p1
    SetProjectionVectorToLine,

    /// Provides for fine control over the effects of the round state variable by
    /// directly setting the values of the three components of the round state:
    /// period, phase, and threshold
    SuperRound,

    /// Sets a new value for reference point 0
    ///
    /// Pops a point number, p, from the stack and sets rp0 to p
    SetReferencePoint0,

    /// Sets a new value for reference point 1
    ///
    /// Pops a point number, p, from the stack and sets rp1 to p
    SetReferencePoint1,

    /// Sets a new value for reference point 2
    ///
    /// Pops a point number, p, from the stack and sets rp2 to p
    SetReferencePoint2,

    /// Establishes a new value for the single width value state variable. The
    /// single width value is used instead of a control value table entry when
    /// the difference between the single width value and the given CVT entry is
    /// less than the single width cut-in
    ///
    //// Pops a 32 bit integer value, n, from the stack and sets the single width
    /// value in the graphics state to n. The value n is expressed in FUnits
    SetSingleWidth,

    /// Establishes a new value for the single width cut-in, the distance difference
    /// at which the interpreter will ignore the values in the control value table
    /// in favor of the single width value
    ///
    /// Pops a 32 bit integer value, n, and sets the single width cut-in to n
    SetSingleWidthCutIn,

    /// Subtracts the number at the top of the stack from the number below it
    ///
    /// Pops two 26.6 numbers, n1 and n2, from the stack and pushes the difference
    /// between the two elements onto the stack
    Subtract,

    /// Sets both the projection vector and freedom vector to the same coordinate
    /// axis causing movement and measurement to be in the same direction. The
    /// setting of the Boolean variable a determines the choice of axis
    SetFreedomAndProjectionVectorsToCoordinateAxis(u8),

    /// Swaps the top two stack elements
    Swap,

    /// Establishes a new value for zp0. It can point to either the glyph zone or the twilight zone
    SetZonePointer0,

    /// Establishes a new value for zp1. It can point to either the glyph zone or the twilight zone
    SetZonePointer1,

    /// Establishes a new value for zp2. It can point to either the glyph zone or the twilight zone
    SetZonePointer2,

    /// Sets all three zone pointers to refer to either the glyph zone or the twilight zone
    SetZonePointerS,

    /// Marks a point as untouched thereby causing the IUP[ ] instruction to affect its location
    UnTouchPoint,

    /// Writes a scaled F26Dot6 value to the specified control value table location
    ///
    /// Pops an integer value, n, and a control value table location l from the
    /// stack. The FUnit value is scaled to the current point size and resolution
    /// and put in the control value table. This instruction assumes the value
    /// is expressed in FUnits and not pixels
    WriteControlValueTableInFunits,

    /// Writes the value in pixels into the control value table location specified
    ///
    /// Pops a value v and a control value table location l from the stack and
    /// puts that value in the specified location in the control value table.
    /// This instruction assumes the value taken from the stack is in pixels and
    /// not in FUnits. The value is written to the CVT table unchanged. The
    /// location l must be less than the number of storage locations specified in
    /// the 'maxp' table in the font file
    WriteControlValueTableInPixels,

    /// Write the value taken from the stack to the specified storage area location
    ///
    /// Pops a storage area location l, followed by a value, v. Writes this 32-bit
    /// value into the storage area location indexed by l. The value must be less
    /// than the number of storage locations specified in the 'maxp' table of the
    /// font file
    WriteStore,
}
