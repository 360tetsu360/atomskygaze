#![allow(dead_code)]
pub(crate) const CONST_NUM: usize = 88;
const SHORT_NAME: [&str; CONST_NUM] = ["Aql","And","Scl","Ara","Lib","Cet","Ari","Sct","Pyx","Boo","Cae","Cha","Cnc","Cap","Car","Cas","Cen","Cep","Com","Cvn","Aur","Col","Cir","Crt","CrA","CrB","Crv","Cru","Cyg","Del","Dor","Dra","Nor","Eri","Sge","For","Gem","Cam","CMa","UMa","Gru","Her","Hor","Hya","Hyi","Ind","Lac","Mon","Lep","Leo","Lup","Lyn","Lyr","Ant","Mic","Mus","Oct","Aps","Oph","Ori","Pav","Peg","Pic","Per","Equ","CMi","LMi","Vul","UMi","Phe","Psc","PsA","Vol","Pup","Ret","Sgr","Sco","Ser","Sex","Men","Tau","Tel","Tuc","Tri","Tra","Aqr","Vir","Vel",];
const SHORT_LONG_MAP: [(&str, &str); CONST_NUM] = [
    ("And", "Andromeda"),
    ("Ant", "Antlia"),
    ("Aps", "Apus"),
    ("Aqr", "Aquarius"),
    ("Aql", "Aquila"),
    ("Ara", "Ara"),
    ("Ari", "Aries"),
    ("Aur", "Auriga"),
    ("Boo", "Bootes"),
    ("Cae", "Caelum"),
    ("Cam", "Camelopardalis"),
    ("Cnc", "Cancer"),
    ("CVn", "Canes Venatici"),
    ("CMa", "Canis Major"),
    ("CMi", "Canis Minor"),
    ("Cap", "Capricornus"),
    ("Car", "Carina"),
    ("Cas", "Cassiopeia"),
    ("Cen", "Centaurus"),
    ("Cep", "Cepheus"),
    ("Cet", "Cetus"),
    ("Cha", "Chamaeleon"),
    ("Cir", "Circinus"),
    ("Col", "Columba"),
    ("Com", "Coma Berenices"),
    ("CrA", "Corona Austrina"),
    ("CrB", "Corona Borealis"),
    ("Crv", "Corvus"),
    ("Crt", "Crater"),
    ("Cru", "Crux"),
    ("Cyg", "Cygnus"),
    ("Del", "Delphinus"),
    ("Dor", "Dorado"),
    ("Dra", "Draco"),
    ("Equ", "Equuleus"),
    ("Eri", "Eridanus"),
    ("For", "Fornax"),
    ("Gem", "Gemini"),
    ("Gru", "Grus"),
    ("Her", "Hercules"),
    ("Hor", "Horologium"),
    ("Hya", "Hydra"),
    ("Hyi", "Hydrus"),
    ("Ind", "Indus"),
    ("Lac", "Lacerta"),
    ("Leo", "Leo"),
    ("LMi", "Leo Minor"),
    ("Lep", "Lepus"),
    ("Lib", "Libra"),
    ("Lup", "Lupus"),
    ("Lyn", "Lynx"),
    ("Lyr", "Lyra"),
    ("Men", "Mensa"),
    ("Mic", "Microscopium"),
    ("Mon", "Monoceros"),
    ("Mus", "Musca"),
    ("Nor", "Norma"),
    ("Oct", "Octans"),
    ("Oph", "Ophiuchus"),
    ("Ori", "Orion"),
    ("Pav", "Pavo"),
    ("Peg", "Pegasus"),
    ("Per", "Perseus"),
    ("Phe", "Phoenix"),
    ("Pic", "Pictor"),
    ("Psc", "Pisces"),
    ("PsA", "Piscis Austrinus"),
    ("Pup", "Puppis"),
    ("Pyx", "Pyxis"),
    ("Ret", "Reticulum"),
    ("Sge", "Sagitta"),
    ("Sgr", "Sagittarius"),
    ("Sco", "Scorpius"),
    ("Scl", "Sculptor"),
    ("Sct", "Scutum"),
    ("Ser", "Serpens"),
    ("Sex", "Sextans"),
    ("Tau", "Taurus"),
    ("Tel", "Telescopium"),
    ("Tri", "Triangulum"),
    ("TrA", "Triangulum Australe"),
    ("Tuc", "Tucana"),
    ("UMa", "Ursa Major"),
    ("UMi", "Ursa Minor"),
    ("Vel", "Vela"),
    ("Vir", "Virgo"),
    ("Vol", "Volans"),
    ("Vul", "Vulpecula"),
];
pub(crate) const CONSTELLATION_NLINES: [u32; 88] = [8,5,3,7,5,20,3,5,2,9,2,2,5,9,14,4,16,6,2,1,6,6,2,9,9,6,6,2,9,5,5,14,5,27,4,1,16,5,17,19,9,18,2,18,4,3,6,6,14,10,13,7,5,1,2,4,3,2,7,21,13,13,2,10,4,1,4,1,7,11,19,6,7,7,4,24,12,11,1,1,12,1,3,3,3,14,12,10,];
pub(crate) const CONSTELLATION_LINES_0: [u32; 16] = [582,579,579,576,579,568,568,580,591,580,568,555,555,551,568,556];
pub(crate) const CONSTELLATION_LINES_1: [u32; 10] = [0,11,11,24,47,24,24,17,17,15];
pub(crate) const CONSTELLATION_LINES_2: [u32; 6] = [685,18,18,681,681,685];
pub(crate) const CONSTELLATION_LINES_3: [u32; 14] = [516,496,496,480,480,475,475,494,494,491,491,490,490,516];
pub(crate) const CONSTELLATION_LINES_4: [u32; 10] = [444,432,432,419,419,412,412,416,416,432];
pub(crate) const CONSTELLATION_LINES_5: [u32; 40] = [50,57,35,13,13,4,13,23,23,27,27,37,37,55,55,62,62,68,68,59,59,35,53,62,53,61,61,67,67,78,78,77,77,69,69,57,57,60,60,67];
pub(crate) const CONSTELLATION_LINES_6: [u32; 6] = [72,48,48,43,43,39];
pub(crate) const CONSTELLATION_LINES_7: [u32; 10] = [537,538,538,542,542,529,529,531,531,537];
pub(crate) const CONSTELLATION_LINES_8: [u32; 4] = [256,261,261,267];
pub(crate) const CONSTELLATION_LINES_9: [u32; 18] = [404,395,395,408,408,418,418,415,415,400,400,399,399,395,395,387,387,384];
pub(crate) const CONSTELLATION_LINES_10: [u32; 4] = [120,126,126,127];
pub(crate) const CONSTELLATION_LINES_11: [u32; 4] = [248,314,314,352];
pub(crate) const CONSTELLATION_LINES_12: [u32; 10] = [264,260,260,249,260,262,262,247,262,269];
pub(crate) const CONSTELLATION_LINES_13: [u32; 18] = [592,593,593,611,611,619,619,625,625,630,619,622,622,611,593,603,611,608];
pub(crate) const CONSTELLATION_LINES_14: [u32; 28] = [277,302,302,316,316,317,317,325,325,321,321,311,311,305,305,279,258,250,250,197,276,279,276,258,197,203,250,242];
pub(crate) const CONSTELLATION_LINES_15: [u32; 8] = [42,28,28,16,16,12,12,1];
pub(crate) const CONSTELLATION_LINES_16: [u32; 32] = [403,391,391,382,382,388,388,389,389,386,386,385,385,380,380,377,385,393,386,401,401,414,388,365,365,356,356,346,346,335,335,336];
pub(crate) const CONSTELLATION_LINES_17: [u32; 12] = [642,666,666,623,623,617,617,642,666,686,686,623];
pub(crate) const CONSTELLATION_LINES_18: [u32; 4] = [374,375,375,355];
pub(crate) const CONSTELLATION_LINES_19: [u32; 2] = [362,371];
pub(crate) const CONSTELLATION_LINES_20: [u32; 12] = [180,179,179,148,148,140,140,138,153,138,153,180];
pub(crate) const CONSTELLATION_LINES_21: [u32; 12] = [193,190,190,177,177,171,171,178,171,163,163,155];
pub(crate) const CONSTELLATION_LINES_22: [u32; 4] = [406,425,406,420];
pub(crate) const CONSTELLATION_LINES_23: [u32; 18] = [322,327,327,332,332,330,330,322,330,331,331,337,337,345,345,338,338,332];
pub(crate) const CONSTELLATION_LINES_24: [u32; 18] = [534,546,546,549,549,557,557,560,560,561,561,559,559,553,553,545,534,530];
pub(crate) const CONSTELLATION_LINES_25: [u32; 12] = [428,427,427,429,429,434,434,439,439,447,447,451];
pub(crate) const CONSTELLATION_LINES_26: [u32; 12] = [359,357,357,351,351,348,348,347,348,363,363,357];
pub(crate) const CONSTELLATION_LINES_27: [u32; 4] = [358,354,368,349];
pub(crate) const CONSTELLATION_LINES_28: [u32; 18] = [563,570,570,575,575,594,594,600,594,604,604,613,613,627,594,583,583,571];
pub(crate) const CONSTELLATION_LINES_29: [u32; 10] = [596,597,597,599,599,605,605,601,601,597];
pub(crate) const CONSTELLATION_LINES_30: [u32; 10] = [166,173,173,159,159,166,159,121,121,111];
pub(crate) const CONSTELLATION_LINES_31: [u32; 28] = [511,513,513,492,492,498,498,511,511,562,562,578,578,522,522,482,482,462,462,452,452,426,426,392,392,361,361,333];
pub(crate) const CONSTELLATION_LINES_32: [u32; 10] = [456,460,460,463,463,453,453,460,453,456];
pub(crate) const CONSTELLATION_LINES_33: [u32; 54] = [33,44,44,52,52,56,56,64,64,66,66,76,76,87,87,99,99,101,101,113,113,116,116,122,122,97,97,92,92,86,86,79,79,70,70,75,75,85,85,91,91,94,94,124,124,130,130,134,134,142,142,143,143,125];
pub(crate) const CONSTELLATION_LINES_34: [u32; 8] = [574,577,577,573,577,584,584,589];
pub(crate) const CONSTELLATION_LINES_35: [u32; 2] = [71,84];
pub(crate) const CONSTELLATION_LINES_36: [u32; 32] = [202,219,219,228,228,227,227,206,228,235,235,239,235,240,235,230,230,222,222,234,222,210,222,204,204,199,204,195,195,189,189,183];
pub(crate) const CONSTELLATION_LINES_37: [u32; 10] = [90,104,104,136,90,102,102,136,102,151];
pub(crate) const CONSTELLATION_LINES_38: [u32; 34] = [211,218,218,213,213,205,205,217,217,220,220,224,224,229,215,216,216,220,216,212,212,201,201,200,201,194,201,205,215,208,192,215,213,211];
pub(crate) const CONSTELLATION_LINES_39: [u32; 38] = [383,378,378,369,369,350,350,324,324,323,323,344,344,350,344,341,341,326,326,306,326,308,323,293,293,288,288,273,288,270,293,291,291,253,253,286,286,324];
pub(crate) const CONSTELLATION_LINES_40: [u32; 18] = [675,650,650,638,638,659,659,677,677,675,659,672,659,664,638,635,635,632];
pub(crate) const CONSTELLATION_LINES_41: [u32; 36] = [503,512,512,489,489,487,487,486,486,471,471,467,467,459,459,443,471,470,470,465,465,461,470,481,481,493,493,485,508,514,514,517,514,508,481,486];
pub(crate) const CONSTELLATION_LINES_42: [u32; 4] = [109,65,65,80];
pub(crate) const CONSTELLATION_LINES_43: [u32; 36] = [259,255,255,254,254,265,265,266,266,259,266,268,268,278,278,287,287,284,284,283,283,292,292,301,301,309,309,319,319,334,334,343,343,373,373,376];
pub(crate) const CONSTELLATION_LINES_44: [u32; 8] = [7,98,98,63,63,54,54,45];
pub(crate) const CONSTELLATION_LINES_45: [u32; 6] = [618,598,598,609,609,618];
pub(crate) const CONSTELLATION_LINES_46: [u32; 12] = [643,652,652,651,651,648,648,647,647,654,654,651];
pub(crate) const CONSTELLATION_LINES_47: [u32; 12] = [188,198,198,223,223,196,196,186,223,245,245,237];
pub(crate) const CONSTELLATION_LINES_48: [u32; 28] = [184,176,176,167,167,158,158,145,158,172,172,165,165,154,154,141,158,154,145,149,145,146,141,145,146,144,149,150];
pub(crate) const CONSTELLATION_LINES_49: [u32; 20] = [342,329,329,300,300,297,297,307,307,328,328,342,307,304,304,294,294,290,328,329];
pub(crate) const CONSTELLATION_LINES_50: [u32; 26] = [442,455,455,449,449,442,449,431,431,423,423,424,423,413,431,433,433,417,417,405,417,402,405,397,405,413];
pub(crate) const CONSTELLATION_LINES_51: [u32; 14] = [281,280,280,274,274,271,271,251,251,231,231,214,214,191];
pub(crate) const CONSTELLATION_LINES_52: [u32; 10] = [532,535,535,539,539,550,550,541,541,535];
pub(crate) const CONSTELLATION_LINES_53: [u32; 2] = [310,296];
pub(crate) const CONSTELLATION_LINES_54: [u32; 4] = [616,610,610,607];
pub(crate) const CONSTELLATION_LINES_55: [u32; 8] = [367,339,339,360,360,364,364,367];
pub(crate) const CONSTELLATION_LINES_56: [u32; 6] = [626,661,661,398,398,626];
pub(crate) const CONSTELLATION_LINES_57: [u32; 4] = [410,466,466,472];
pub(crate) const CONSTELLATION_LINES_58: [u32; 14] = [500,506,483,506,500,479,479,458,458,469,469,483,483,495];
pub(crate) const CONSTELLATION_LINES_59: [u32; 42] = [164,161,161,157,182,187,187,185,185,174,187,181,181,175,175,164,164,169,169,147,147,157,157,152,152,160,160,175,152,131,131,133,133,135,135,139,131,132,132,137,185,181];
pub(crate) const CONSTELLATION_LINES_60: [u32; 26] = [595,621,621,602,602,590,590,595,590,586,586,533,533,547,547,590,547,540,540,524,524,518,518,540,518,507];
pub(crate) const CONSTELLATION_LINES_61: [u32; 26] = [3,674,673,660,660,639,673,667,667,662,662,637,637,629,674,663,663,658,658,641,641,628,0,673,0,3];
pub(crate) const CONSTELLATION_LINES_62: [u32; 4] = [207,170,170,168];
pub(crate) const CONSTELLATION_LINES_63: [u32; 20] = [96,103,103,107,107,105,105,93,93,88,88,81,81,74,88,83,83,82,82,73];
pub(crate) const CONSTELLATION_LINES_64: [u32; 8] = [612,614,614,620,620,615,615,612];
pub(crate) const CONSTELLATION_LINES_65: [u32; 2] = [236,232];
pub(crate) const CONSTELLATION_LINES_66: [u32; 8] = [320,312,312,298,298,289,298,320];
pub(crate) const CONSTELLATION_LINES_67: [u32; 2] = [569,587];
pub(crate) const CONSTELLATION_LINES_68: [u32; 14] = [58,497,497,473,473,435,435,457,457,422,422,411,411,435];
pub(crate) const CONSTELLATION_LINES_69: [u32; 22] = [22,21,21,8,8,22,21,31,31,41,41,21,21,29,29,8,8,9,9,2,2,8];
pub(crate) const CONSTELLATION_LINES_70: [u32; 38] = [19,25,19,26,26,25,25,32,32,36,36,46,46,40,40,34,34,30,30,20,20,14,14,6,6,689,689,687,687,688,688,683,683,679,679,684,684,687];
pub(crate) const CONSTELLATION_LINES_71: [u32; 12] = [671,657,657,633,633,631,631,640,640,655,655,670];
pub(crate) const CONSTELLATION_LINES_72: [u32; 14] = [238,221,221,244,244,238,244,225,244,252,252,272,272,244];
pub(crate) const CONSTELLATION_LINES_73: [u32; 14] = [243,241,241,226,226,203,203,209,209,233,233,242,242,243];
pub(crate) const CONSTELLATION_LINES_74: [u32; 8] = [110,112,112,106,106,95,95,110];
pub(crate) const CONSTELLATION_LINES_75: [u32; 48] = [521,527,520,525,525,515,515,509,515,521,521,525,525,552,552,536,536,521,536,527,527,519,552,558,558,543,543,536,543,548,548,554,554,564,564,565,558,572,572,588,588,585,585,581,581,567,581,566];
pub(crate) const CONSTELLATION_LINES_76: [u32; 24] = [499,505,505,510,510,501,501,484,484,478,478,477,477,476,476,468,468,464,464,450,464,448,464,454];
pub(crate) const CONSTELLATION_LINES_77: [u32; 22] = [440,441,441,436,436,430,430,437,437,446,446,438,438,437,544,523,523,504,504,502,502,488];
pub(crate) const CONSTELLATION_LINES_78: [u32; 2] = [313,299];
pub(crate) const CONSTELLATION_LINES_79: [u32; 2] = [156,129];
pub(crate) const CONSTELLATION_LINES_80: [u32; 24] = [153,128,128,118,123,162,114,115,114,108,108,89,123,118,123,119,119,114,118,117,117,115,115,100];
pub(crate) const CONSTELLATION_LINES_81: [u32; 2] = [528,526];
pub(crate) const CONSTELLATION_LINES_82: [u32; 6] = [645,680,680,5,680,10];
pub(crate) const CONSTELLATION_LINES_83: [u32; 6] = [51,49,49,38,38,51];
pub(crate) const CONSTELLATION_LINES_84: [u32; 6] = [474,421,421,445,445,474];
pub(crate) const CONSTELLATION_LINES_85: [u32; 28] = [624,634,634,646,646,649,649,656,656,668,668,678,678,682,634,644,644,636,644,653,653,665,665,669,669,676,606,624];
pub(crate) const CONSTELLATION_LINES_86: [u32; 24] = [340,353,353,366,366,379,379,394,394,396,396,407,379,381,381,390,390,409,381,370,370,372,370,366];
pub(crate) const CONSTELLATION_LINES_87: [u32; 20] = [246,257,257,263,263,282,282,295,295,318,318,315,315,303,303,285,285,275,275,246];
pub(crate) const CONSTELLATION_LINES: [&[u32]; CONST_NUM] = include!("const_lines");
const STARS_N: usize = 690;
pub(crate) const STAR_POSITIONS: [f64;STARS_N*2] = include!("const_data");
