#[pdf_enum]
pub(crate) enum PredefinedCjkCmapName {
    // chinese (simplified)
    /// Microsoft Code Page 936 (lfCharSet 0x86), GB 2312-80 character set, EUC-CN encoding
    GbEucH = "GB-EUC-H",

    /// Vertical version of GB-EUC-H
    GbEucV = "GB-EUC-V",

    /// Mac OS, GB 2312-80 character set, EUC-CN encoding, Script Manager code 19
    GBpcEucH = "GBpc-EUC-H",

    /// Vertical version of GBpc-EUC-H
    GBpcEucV = "GBpc-EUC-V",

    /// Microsoft Code Page 936 (lfCharSet 0x86), GBK character set, GBK encoding
    GbkEucH = "GBK-EUC-H",

    /// Vertical version of GBK-EUC-H
    GbkEucV = "GBK-EUC-V",

    /// Same as GBK-EUC-H but replaces half-width Latin characters with proportional forms and maps character code 0x24 to a dollar sign ($) instead of a yuan symbol (¥)
    GbKpEucH = "GBKp-EUC-H",

    /// Vertical version of GBKp-EUC-H
    GbKpEucV = "GBKp-EUC-V",

    /// GB 18030-2000 character set, mixed 1-, 2-, and 4-byte encoding
    Gbk2KH = "GBK2K-H",

    /// Vertical version of GBK2K-H
    Gbk2KV = "GBK2K-V",

    /// Unicode (UCS-2) encoding for the Adobe-GB1 character collection
    UniGbUcs2H = "UniGB-UCS2-H",

    /// Vertical version of UniGB-UCS2-H
    UniGbUcs2V = "UniGB-UCS2-V",

    /// Unicode (UTF-16BE) encoding for the Adobe-GB1 character collection; contains mappings for all characters in the GB18030-2000 character set
    UniGbUtf16H = "UniGB-UTF16-H",

    /// Vertical version of UniGB-UTF16-H
    UniGbUtf16V = "UniGB-UTF16-V",

    // Chinese (Traditional)
    /// Mac OS, Big Five character set, Big Five encoding, Script Manager code 2
    B5pcH = "B5pc-H",

    /// Vertical version of B5pc-H
    B5pcV = "B5pc-V",

    /// Hong Kong SCS, an extension to the Big Five character set and encoding
    HKscsB5H = "HKscs-B5-H",

    /// Vertical version of HKscs-B5-H
    HKscsB5V = "HKscs-B5-V",

    /// Microsoft Code Page 950 (lfCharSet 0x88), Big Five character set with ETen extensions
    ETenB5H = "ETen-B5-H",

    /// Vertical version of ETen-B5-H
    ETenB5V = "ETen-B5-V",

    /// Same as ETen-B5-H but replaces half-width Latin characters with proportional forms
    ETenmsB5H = "ETenms-B5-H",

    /// Vertical version of ETenms-B5-H
    ETenmsB5V = "ETenms-B5-V",

    /// CNS 11643-1992 character set, EUC-TW encoding
    CnsEucH = "CNS-EUC-H",

    /// Vertical version of CNS-EUC-H
    CnsEucV = "CNS-EUC-V",

    /// Unicode (UCS-2) encoding for the Adobe-CNS1 character collection
    UniCnsUcs2H = "UniCNS-UCS2-H",

    /// Vertical version of UniCNS-UCS2-H
    UniCnsUcs2V = "UniCNS-UCS2-V",

    /// Unicode (UTF-16BE) encoding for the Adobe-CNS1 character collection; contains mappings for all the characters in the HKSCS-2001 character set and contains both 2- and 4-byte character codes
    UniCnsUtf16H = "UniCNS-UTF16-H",

    /// Vertical version of UniCNS-UTF16-H
    UniCnsUtf16V = "UniCNS-UTF16-V",

    // japanese
    /// Mac OS, JIS X 0208 character set with KanjiTalk6 extensions, Shift-JIS encoding, Script Manager code 1
    _83pvRksjH = "83pv-RKSJ-H",

    /// Microsoft Code Page 932 (lfCharSet 0x80), JIS X 0208 character set with NEC and IBM® extensions
    _90msRksjH = "90ms-RKSJ-H",

    /// Vertical version of 90ms-RKSJ-H
    _90msRksjV = "90ms-RKSJ-V",

    /// Same as 90ms-RKSJ-H but replaces half-width Latin characters with proportional forms
    _90mspRksjH = "90msp-RKSJ-H",

    /// Vertical version of 90msp-RKSJ-H
    _90mspRksjV = "90msp-RKSJ-V",

    /// Mac OS, JIS X 0208 character set with KanjiTalk7 extensions, Shift-JIS encoding, Script Manager code 1
    _90pvRksjH = "90pv-RKSJ-H",

    /// JIS X 0208 character set with Fujitsu FMR extensions, Shift-JIS encoding
    AddRksjH = "Add-RKSJ-H",

    /// Vertical version of Add-RKSJ-H
    AddRksjV = "Add-RKSJ-V",

    /// JIS X 0208 character set, EUC-JP encoding
    EucH = "EUC-H",

    /// Vertical version of EUC-H
    EucV = "EUC-V",

    /// JIS C 6226 (JIS78) character set with NEC extensions, Shift-JIS encoding
    ExtRksjH = "Ext-RKSJ-H",

    /// Vertical version of Ext-RKSJ-H
    ExtRksjV = "Ext-RKSJ-V",

    /// JIS X 0208 character set, ISO-2022-JP encoding
    H = "H",

    /// Vertical version of H
    V = "V",

    /// Unicode (UCS-2) encoding for the Adobe-Japan1 character collection
    UniJisUcs2H = "UniJIS-UCS2-H",

    /// Vertical version of UniJIS-UCS2-H
    UniJisUcs2V = "UniJIS-UCS2-V",

    /// Same as UniJIS-UCS2-H but replaces proportional Latin characters with half-width forms
    UniJisUcs2HwH = "UniJIS-UCS2-HW-H",

    /// Vertical version of UniJIS-UCS2-HW-H
    UniJisUcs2HwV = "UniJIS-UCS2-HW-V",

    /// Unicode (UTF-16BE) encoding for the Adobe-Japan1 character collection; contains mappings for all characters in the JIS X 0213:1000 character set
    UniJisUtf16H = "UniJIS-UTF16-H",

    /// Vertical version of UniJIS-UTF16-H
    UniJisUtf16V = "UniJIS-UTF16-",

    // korean
    /// KS X 1001:1992 character set, EUC-KR encoding
    KscEucH = "KSC-EUC-H",

    /// Vertical version of KSC-EUC-H
    KscEucV = "KSC-EUC-V",

    /// Microsoft Code Page 949 (lfCharSet 0x81), KS X 1001:1992 character set plus 8822 additional hangul, Unified Hangul Code (UHC) encoding
    KsCmsUhcH = "KSCms-UHC-H",

    /// Vertical version of KSCms−UHC-H
    KsCmsUhcV = "KSCms-UHC-V",

    /// Same as KSCms-UHC-H but replaces proportional Latin characters with half-width forms
    KsCmsUhcHwH = "KSCms-UHC-HW-H",

    /// Vertical version of KSCms-UHC-HW-H
    KsCmsUhcHwV = "KSCms-UHC-HW-V",

    /// Mac OS, KS X 1001:1992 character set with Mac OS KH extensions, Script Manager Code 3
    KsCpcEucH = "KSCpc-EUC-H",

    /// Unicode (UCS-2) encoding for the Adobe-Korea1 character collection
    UniKsUcs2H = "UniKS-UCS2-H",

    /// Vertical version of UniKS-UCS2-H
    UniKsUcs2V = "UniKS-UCS2-V",

    /// Unicode (UTF-16BE) encoding for the Adobe-Korea1 character collection
    UniKsUtf16H = "UniKS-UTF16-H",

    /// Vertical version of UniKS-UTF16-H
    UniKsUtf16V = "UniKS-UTF16-",

    // generic
    /// The horizontal identity mapping for 2-byte CIDs; may be used with CIDFonts using any Registry, Ordering, and Supplement values. It maps 2-byte character codes ranging from 0 to 65,535 to the same 2-byte CID value, interpreted high-order byte first.
    IdentityH = "Identity-H",

    /// Vertical version of Identity-H. The mapping is the same as for Identity-H.
    IdentityV = "Identity-V",
}
