#[derive(Debug, Clone, Copy)]
pub(super) enum GraphicsOperator {
    // Starting and finishing
    /// Finishes a charstring outline definition and must be the last command
    /// in a character’s outline (except for accented characters defined using
    /// seac). When endchar is executed, Type 1 BuildChar performs several tasks.
    ///
    /// It executes a setcachedevice operation, using a bounding box it computes
    /// directly from the character outline and using the width information
    /// acquired from a previous hsbw or sbw operation. (Note that this is not
    /// the same order of events as in Type 3 Fonts.) BuildChar then calls a
    /// special version of fill or stroke depending on the value of PaintType
    /// in the font dictionary. The Type 1 font format supports only PaintType
    /// 0 (fill) and 2 (outline). Note that this single fill or stroke implies
    /// that there can be only one path (possibly containing several subpaths)
    /// that can be created to be filled or stroked by the endchar command
    EndChar,

    /// Sets the left sidebearing point at (sbx, 0) and sets the character width
    /// vector to (wx, 0) in character space. This command also sets the current
    /// point to (sbx, 0), but does not place the point in the character path.
    ///
    /// Use rmoveto for the first point in the path. The name hsbw stands for
    /// horizontal sidebearing and width; horizontal indicates that the y component
    /// of both the sidebearing and width is 0. Either sbw or hsbw must be used
    /// once as the first command in a character outline definition. It must be
    /// used only once. In non-marking characters, such as the space character,
    /// the left sidebearing point should be (0, 0)
    HorizontalSideBearingWidth,

    /// makes an accented character from two other characters in its font program.
    ///
    /// The asb argument is the x component of the left sidebearing of the accent;
    /// this value must be the same as the sidebearing value given in the hsbw
    /// or sbw command in the accent’s own charstring. The origin of the accent
    /// is placed at (adx, ady) relative to the origin of the base character. The
    /// bchar argument is the character code of the base character, and the achar
    /// argument is the character code of the accent character. Both bchar and
    /// achar are codes that these characters are assigned in the Adobe StandardEncoding
    /// vector, given in an Appendix in the PostScript Language Reference Manual.
    ///
    /// Furthermore, the characters represented by achar and bchar must be in the
    /// same positions in the font’s encoding vector as the positions they occupy
    /// in the Adobe StandardEncoding vector. If the name of both components of
    /// an accented character do not appear in the Adobe StandardEncoding vector,
    /// the accented character cannot be built using the seac command
    ///
    /// The FontBBox entry in the font dictionary must be large enough to accommodate
    /// both parts of the accented character. The sbw or hsbw command that begins
    /// the accented character must be the same as the corresponding command in
    /// the base character. Finally, seac is the last command in the charstring
    /// for the accented character because the accent and base characters’ charstrings
    /// each already end with their own endchar commands
    ///
    /// The use of this command saves space in a Type 1 font program, but its use
    /// is restricted to those characters whose parts are defined in the Adobe
    /// StandardEncoding vector. In situations where use of the seac command is
    /// not possible, use of Subrs subroutines is a more general means for creating
    /// accented characters
    StandardEncodingAccentedCharacter,

    /// sets the left sidebearing point to (sbx, sby) and sets the character
    /// width vector to (wx, wy) in character space. This command also sets the
    /// current point to (sbx, sby), but does not place the point in the character
    /// path. Use rmoveto for the first point in the path. The name sbw stands
    /// for sidebearing and width; the x and y components of both the left
    /// sidebearing and width must be specified. If the y components of both the
    /// left sidebearing and the width are 0, then the hsbw command should be used.
    ///
    /// Either sbw or hsbw must be used once as the first command in a character
    /// outline definition. It must be used only once
    SideBearingWidth,

    // Path construction
    /// `closepath` closes a subpath. Adobe strongly recommends that all character
    /// subpaths end with a `closepath` command, otherwise when an outline is stroked
    /// (by setting PaintType equal to 2) you may get unexpected behavior where
    /// lines join. Note that, unlike the `closepath` command in the PostScript
    /// language, this command does not reposition the current point. Any subsequent
    /// rmoveto must be relative to the current point in force before the Type
    /// 1 font format `closepath` command was given. Make sure that any subpath
    /// section formed by the `closepath` command intended to be zero length, is
    /// zero length. If not, the `closepath` command may cause a “spike” or “hangnail”
    /// (if the subpath doubles back onto itself) with unexpected results
    ClosePath,

    /// Equivalent to `dx 0 rlineto`
    HorizontalLineTo,

    /// Equivalent to `dx 0 rmoveto`
    HorizontalMoveTo,

    /// Equivalent to `dx1 0 dx2 dy2 0 dy3 rrcurveto`
    ///
    /// This command eliminates two arguments from an rrcurveto call when the
    /// first Bézier tangent is horizontal and the second Bézier tangent is
    /// vertical
    HorizontalVerticalCurveTo,

    /// appends a straight line segment to the current path, starting from the
    /// current point and extending dx user space units horizontally and dy units
    /// vertically. That is, the operands dx and dy are interpreted as relative
    /// displacements from the current point rather than as absolute coordinates.
    ///
    /// In all other respects, the behavior of rlineto is identical to that of lineto.
    ///
    /// If the current point is undefined because the current path is empty, a
    /// `nocurrentpoint` error occurs
    RelativeLineTo,

    /// starts a new subpath of the current path by displacing the coordinates
    /// of the current point dx user space units horizontally and dy units
    /// vertically, without connecting it to the previous current point. That
    /// is, the operands dx and dy are interpreted as relative displacements
    /// from the current point rather than as absolute coordinates. In all other
    /// respects, the behavior of rmoveto is identical to that of moveto
    ///
    /// If the current point is undefined because the current path is empty, a
    /// `nocurrentpoint` error occurs
    RelativeMoveTo,

    /// Whereas the arguments to the rcurveto operator in the PostScript language
    /// are all relative to the current point, the arguments to rrcurveto are
    /// relative to each other.
    ///
    /// Equivalent to `dx1 dy1 (dx1+dx2) (dy1+dy2) (dx1+dx2+dx3) (dy1+dy2+dy3) rcurveto`
    ///
    /// `rcurveto` docs:
    /// appends a section of a cubic Bézier curve to the current path in the same
    /// manner as curveto. However, the operands are interpreted as relative
    /// displacements from the current point rather than as absolute coordinates.
    /// That is, rcurveto constructs a curve between the current point (x0, y0)
    /// and the endpoint (x0 + dx3, y0 + dy3), using (x0 + dx1, y0 + dy1) and
    /// (x0 + dx2, y0 + dy2) as the Bézier control points. In all other respects,
    /// the behavior of rcurveto is identical to that of curveto
    ///
    /// `curveto` docs:
    /// appends a section of a cubic Bézier curve to the current path between the
    /// current point (x0, y0) and the endpoint (x3, y3), using (x1, y1) and (x2,
    /// y2) as the Bézier control points. The endpoint (x3, y3) becomes the new
    /// current point. If the current point is undefined because the current path
    /// is empty, a nocurrentpoint error occurs.
    RelativeRelativeCurveTo,

    /// Equivalent to `0 dy1 dx2 dy2 dx3 0 rrcurveto`.
    ///
    /// This command eliminates two arguments from an `rrcurveto` call when the
    /// first Bézier tangent is vertical and the second Bézier tangent is
    /// horizontal
    VerticalHorizontalCurveTo,

    /// Equivalent to `0 dy rlineto`
    VerticalLineTo,

    /// Equivalent to `0 dy rmoveto`
    VerticalMoveTo,

    // Hint commands
    /// brackets an outline section for the dots in letters such as “i”,“ j”,
    /// and “!”. This is a hint command that indicates that a section of a charstring
    /// should be understood as describing such a feature, rather than as part
    /// of the main outline
    DotSection,

    /// declares the vertical range of a horizontal stem zone between the y
    /// coordinates y and y+dy, where y is relative to the y coordinate of the
    /// left sidebearing point. Horizontal stem zones within a set of stem hints
    /// for a single character may not overlap other horizontal stem zones. Use
    /// hint replacement to avoid stem hint overlaps
    HorizontalStem,

    /// declares the vertical ranges of three horizontal stem zones between the
    /// y coordinates `y0` and `y0 + dy0`, `y1` and `y1 + dy1`, and between `y2`
    /// and `y2 + dy2`, where `y0`, `y1` and `y2` are all relative to the y
    /// coordinate of the left sidebearing point. The hstem3 command sorts these
    /// zones by the y values to obtain the lowest, middle and highest zones,
    /// called ymin, ymid and ymax respectively. The corresponding dy values are
    /// called dymin, dymid and dymax. These stems and the counters between them
    /// will all be controlled. These coordinates must obey certain restrictions:
    ///
    ///     - dymin = dymax
    ///
    ///     - The distance from ymin + dymin/2 to ymid + dymid/2 must equal the
    ///       distance from ymid + dymid/2 to ymax + dymax/2. In other words,
    ///       the distance from the center of the bottom stem to the center of
    ///       the middle stem must be the same as the distance from the center
    ///       of the middle stem to the center of the top stem.
    ///
    /// If a charstring uses an hstem3 command in the hints for a character, the
    /// charstring must not use hstem commands and it must use the same hstem3
    /// command consistently if hint replacement is performed.
    ///
    /// The hstem3 command is especially suited for controlling the stems and
    /// counters of symbols with three horizontally oriented features with equal
    /// vertical widths and with equal white space between these features, such
    /// as the mathematical equivalence symbol or the division symbol.
    HorizontalStem3,

    /// declares the horizontal range of a vertical stem zone between the x
    /// coordinates x and x+dx, where x is relative to the x coordinate of the
    /// left sidebearing point. Vertical stem zones within a set of stem hints
    /// for a single character may not overlap other vertical stem zones. Use
    /// hint replacement to avoid stem hint overlap
    VerticalStem,

    /// declares the horizontal ranges of three vertical stem zones between the
    /// x coordinates x0 and x0 + dx0, x1 and x1 + dx1, and x2 and x2 + dx2, where
    /// x0, x1 and x2 are all relative to the x coordinate of the left sidebearing
    /// point. The vstem3 command sorts these zones by the x values to obtain the
    /// leftmost, middle and rightmost zones, called xmin, xmid and xmax respectively.
    /// The corresponding dx values are called dxmin, dxmid and dxmax. These stems
    /// and the counters between them will all be controlled. These coordinates
    /// must obey certain restrictions described as follows:
    ///
    ///     - dxmin = dxmax
    ///
    ///     - The distance from xmin + dxmin/2 to xmid + dxmid/2 must equal the
    ///       distance from xmid + dxmid/2 to xmax + dxmax/2. In other words, the
    ///       distance from the center of the left stem to the center of the
    ///       middle stem must be the same as the distance from the center of the
    ///       middle stem to the center of the right stem
    ///
    /// If a charstring uses a vstem3 command in the hints for a character, the
    /// charstring must not use vstem commands and it must use the same vstem3
    /// command consistently if hint replacement is performed
    ///
    /// The vstem3 command is especially suited for controlling the stems and
    /// counters of characters such as a lower case “m.”
    VerticalStem3,

    // Arithmetic
    /// divides `num1` by `num2`, producing a result that is always a real number
    /// even if both operands are integers
    Div,

    // Subroutine
    /// a mechanism used by Type 1 BuildChar to make calls on the PostScript
    /// interpreter. Arguments argn through arg1 are pushed onto the PostScript
    /// interpreter operand stack, and the PostScript language procedure in the
    /// othersubr# position in the OtherSubrs array in the Private dictionary (or
    /// a built-in function equivalent to this procedure) is executed. Note that
    /// the argument order will be reversed when pushed onto the PostScript
    /// interpreter operand stack. After the arguments are pushed onto the
    /// PostScript interpreter operand stack, the PostScript interpreter performs
    /// a begin operation on systemdict followed by a begin operation on the font
    /// dictionary prior to executing the OtherSubrs entry. When the OtherSubrs
    /// entry completes its execution, the PostScript interpreter performs two
    /// end operations prior to returning to Type 1 BuildChar charstring execution.
    ///
    /// Use pop commands to retrieve results from the PostScript operand stack
    /// back to the Type 1 BuildChar operand stack
    CallOtherSubroutine,

    /// calls a charstring subroutine with index subr# from the Subrs array in
    /// the Private dictionary. Each element of the Subrs array is a charstring
    /// encoded and encrypted like any other charstring. Arguments pushed on the
    /// Type 1 BuildChar operand stack prior to calling the subroutine, and results
    /// pushed on this stack by the subroutine, act according to the manner in
    /// which the subroutine is coded. These subroutines are generally used to
    /// encode sequences of path commands that are repeated throughout the font
    /// program, for example, serif outline sequences. Subroutine calls may be
    /// nested 10 deep
    CallSubroutine,

    /// removes a number from the top of the PostScript interpreter operand stack
    /// and pushes that number onto the Type 1 BuildChar operand stack. This
    /// command is used only to retrieve a result from an OtherSubrs procedure
    Pop,

    /// returns from a Subrs array charstring subroutine (that had been called
    /// with a callsubr command) and continues execution in the calling charstring
    Return,

    /// sets the current point in the Type 1 font format BuildChar to (x, y) in
    /// absolute character space coordinates without performing a charstring
    /// moveto command. This establishes the current point for a subsequent relative
    /// path building command. The setcurrentpoint command is used only in
    /// conjunction with results from OtherSubrs procedures
    SetCurrentPoint,
}
