#[pdf_enum]
#[allow(non_camel_case_types)]
pub enum PdfGraphicsOperator {
    /// Close, fill, and stroke path using nonzero winding number rule
    b = "b",

    /// Fill and stroke path using nonzero winding number rule
    B = "B",

    /// Close, fill, and stroke path using even-odd rule
    b_star = "b*",

    /// Fill and stroke path using even-odd rule
    B_star = "B*",

    /// Begin marked-content sequence with property list
    BDC = "BDC",

    /// Begin inline image object
    BI = "BI",

    /// Begin marked-content sequence
    BMC = "BMC",

    /// Begin text object
    BT = "BT",

    /// Begin compatibility section
    BX = "BX",

    /// Append curved segment to path (three control points)
    c = "c",

    /// Concatenate matrix to current transformation matrix
    cm = "cm",

    /// Set color space for stroking operations
    CS = "CS",

    /// Set color space for nonstroking operations
    cs = "cs",

    /// Set line dash pattern
    d = "d",

    /// Set glyph width in Type 3 font
    d0 = "d0",

    /// Set glyph width and bounding box in Type 3 font
    d1 = "d1",

    /// Invoke named XObject
    Do = "Do",

    /// Define marked-content point with property list
    DP = "DP",

    /// End inline image object
    EI = "EI",

    /// End marked-content sequence
    EMC = "EMC",

    /// End text object
    ET = "ET",

    /// End compatibility section
    EX = "EX",

    /// Fill path using nonzero winding number rule
    f = "f",

    /// Fill path using nonzero winding number rule (obsolete)
    F = "F",

    /// Fill path using even-odd rule
    f_star = "f*",

    /// Set gray level for stroking operations
    G = "G",

    /// Set gray level for nonstroking operations
    g = "g",

    /// Set parameters from graphics state parameter dictionary
    gs = "gs",

    /// Close subpath
    h = "h",

    /// Set flatness tolerance
    i = "i",

    /// Begin inline image data
    ID = "ID",

    /// Set line join style
    j = "j",

    /// Set line cap style
    J = "J",

    /// Set CMYK color for stroking operations
    K = "K",

    /// Set CMYK color for nonstroking operations
    k = "k",

    /// Append straight line segment to path
    l = "l",

    /// Begin new subpath
    m = "m",

    /// Set miter limit
    M = "M",

    /// Define marked-content point
    MP = "MP",

    /// End path without filling or stroking
    n = "n",

    /// Save graphics state
    q = "q",

    /// Restore graphics state
    Q = "Q",

    /// Append rectangle to path
    re = "re",

    /// Set RGB color for stroking operations
    RG = "RG",

    /// Set RGB color for nonstroking operations
    rg = "rg",

    /// Set color rendering intent
    ri = "ri",

    /// Close and stroke path
    s = "s",

    /// Stroke path
    S = "S",

    /// Set color for stroking operations
    SC = "SC",

    /// Set color for nonstroking operations
    sc = "sc",

    /// Set color for stroking operations (ICCBased and special colour spaces)
    SCN = "SCN",

    /// Set color for nonstroking operations (ICCBased and special colour spaces)
    scn = "scn",

    /// Paint area defined by shading pattern
    sh = "sh",

    /// Move to start of next text line
    T_star = "T*",

    /// Set character spacing
    Tc = "Tc",

    /// Move text position
    Td = "Td",

    /// Move text position and set leading
    TD = "TD",

    /// Set text font and size
    Tf = "Tf",

    /// Show text
    Tj = "Tj",

    /// Show text, allowing individual glyph positioning
    TJ = "TJ",

    /// Set text leading
    TL = "TL",

    /// Set text matrix and text line matrix
    Tm = "Tm",

    /// Set text rendering mode
    Tr = "Tr",

    /// Set text rise
    Ts = "Ts",

    /// Set word spacing
    Tw = "Tw",

    /// Set horizontal text scaling
    Tz = "Tz",

    /// Append curved segment to path (initial point replicated)
    v = "v",

    /// Set line width
    w = "w",

    /// Set clipping path using nonzero winding number rule
    W = "W",

    /// Set clipping path using even-odd rule
    W_star = "W*",

    /// Append curved segment to path (final point replicated)
    y = "y",

    /// Move to next line and show text
    single_quote = "'",

    /// Set word and character spacing, move to next line, and show text
    double_quote = "\"",
}
