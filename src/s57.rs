use std::collections::HashMap;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Position {
    lat: f64,
    lon: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forward,
    Reverse,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LineElement {
    start_connected_node: u32,
    edge_vector: u32,
    end_connected_node: u32,
    direction: Direction,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PointGeometry {
    position: Position,
    value: f64,
}

pub type MultiGeometry = Vec<Position>;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum AttributeValue {
    UInt32(u32),
    Float(f32),
    String(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VectorEdge {
    points: Vec<Position>,
}

#[allow(dead_code)]
impl VectorEdge {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn set_positions(&mut self, positions: Vec<Position>) {
        self.points = positions;
    }

    pub fn set_points(&mut self, points: &[f32]) {
        for i in 0..points.len() / 2 {
            self.points.push(Position {
                lat: points[i * 2 + 1] as f64,
                lon: points[i * 2] as f64,
            });
        }
    }

    pub fn positions(&self) -> &Vec<Position> {
        &self.points
    }
}

#[derive(Debug, Clone)]
pub struct ConnectedNode {
    position: Position,
}

#[allow(dead_code)]
impl ConnectedNode {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct S57 {
    s57_type: S57Type,
    line_elements: Vec<LineElement>,
    polygon_line_elements: Vec<LineElement>,
    lines: Vec<MultiGeometry>,
    polygons: Vec<MultiGeometry>,
    multi_point_geometry: Vec<PointGeometry>,
    point_geometry: Option<Position>,
    attributes: HashMap<S57Attribute, AttributeValue>,
}
#[allow(dead_code)]
impl S57 {
    pub fn new(s57_type: S57Type) -> Self {
        Self {
            s57_type,
            line_elements: Vec::new(),
            polygon_line_elements: Vec::new(),
            lines: Vec::new(),
            polygons: Vec::new(),
            multi_point_geometry: Vec::new(),
            point_geometry: None,
            attributes: HashMap::new(),
        }
    }

    pub fn from_type_code(type_code: i32) -> Self {
        Self {
            s57_type: S57Type::from_type_code(type_code),
            line_elements: Vec::new(),
            polygon_line_elements: Vec::new(),
            lines: Vec::new(),
            polygons: Vec::new(),
            multi_point_geometry: Vec::new(),
            point_geometry: None,
            attributes: HashMap::new(),
        }
    }

    pub fn set_attribute(&mut self, attribute: S57Attribute, value: AttributeValue) {
        self.attributes.insert(attribute, value);
    }

    pub fn attribute_list(&self) -> Vec<S57Attribute> {
        self.attributes.keys().cloned().collect()
    }

    pub fn attribute(&self, attribute: S57Attribute) -> Option<&AttributeValue> {
        self.attributes.get(&attribute)
    }

    pub fn build_geometry(
        &mut self,
        vector_edges: &HashMap<u32, VectorEdge>,
        connected_nodes: &HashMap<u32, ConnectedNode>,
    ) {
        self.lines = S57::build_geometries(&self.line_elements, vector_edges, connected_nodes);
        self.polygons =
            S57::build_geometries(&self.polygon_line_elements, vector_edges, connected_nodes);
    }

    pub fn set_line_geometry(&mut self, elements: &[LineElement]) {
        self.line_elements = elements.to_vec();
    }

    pub fn set_polygon_geometry(&mut self, elements: &[LineElement]) {
        self.polygon_line_elements = elements.to_vec();
    }

    pub fn set_point_geometry(&mut self, position: Position) {
        self.point_geometry = Some(position);
    }

    pub fn set_multi_point_geometry(&mut self, points: Vec<PointGeometry>) {
        self.multi_point_geometry = points;
    }

    pub fn point_geometry(&self) -> Option<&Position> {
        self.point_geometry.as_ref()
    }

    pub fn multi_point_geometry(&self) -> &Vec<PointGeometry> {
        &self.multi_point_geometry
    }

    pub fn polygons(&self) -> &Vec<MultiGeometry> {
        &self.polygons
    }

    pub fn lines(&self) -> &Vec<MultiGeometry> {
        &self.lines
    }

    pub fn s57_type(&self) -> S57Type {
        self.s57_type
    }

    pub fn build_geometries<T: Clone>(
        line_elements: &[LineElement],
        vector_edges: &HashMap<u32, VectorEdge>,
        connected_nodes: &HashMap<u32, ConnectedNode>,
    ) -> Vec<T>
    where
        Vec<Position>: Into<T>,
    {
        // Find connected line strings
        let mut line_strings: Vec<Vec<LineElement>> = Vec::new();

        for line_element in line_elements {
            let mut found_line_string = false;

            // Try to connect to an existing line string
            for line_string in line_strings.iter_mut() {
                if line_element.start_connected_node
                    == line_string.last().unwrap().end_connected_node
                {
                    // Append to the end of an existing line string
                    line_string.push(line_element.clone());
                    found_line_string = true;
                    break;
                } else if line_element.end_connected_node
                    == line_string.first().unwrap().start_connected_node
                {
                    // Insert at the beginning of an existing line string
                    line_string.insert(0, line_element.clone());
                    found_line_string = true;
                    break;
                }
            }

            // If no connection found, start a new line string
            if !found_line_string {
                line_strings.push(vec![line_element.clone()]);
            }
        }

        // Build geometries from line strings
        let mut geometries = Vec::new();

        for line_string in line_strings {
            let mut geometry: Vec<Position> = Vec::new();

            for line_element in &line_string {
                // Add start node position
                if let Some(start_node) = connected_nodes.get(&line_element.start_connected_node) {
                    geometry.push(start_node.position().clone());
                } else {
                    eprintln!(
                        "Connected node index {} not found",
                        line_element.start_connected_node
                    );
                }

                // Add vector edge points if edge exists
                if line_element.edge_vector != 0 {
                    if let Some(vector_edge) = vector_edges.get(&line_element.edge_vector) {
                        let positions = vector_edge.positions();
                        match line_element.direction {
                            Direction::Reverse => geometry.extend(positions.iter().rev().cloned()),
                            Direction::Forward => geometry.extend(positions.iter().cloned()),
                        }
                    } else {
                        eprintln!("Vector edge {} not found", line_element.edge_vector);
                    }
                }
            }

            // Add end node position of the last line element
            if let Some(end_node) =
                connected_nodes.get(&line_string.last().unwrap().end_connected_node)
            {
                geometry.push(end_node.position().clone());
            } else {
                eprintln!(
                    "Connected node index {} not found",
                    line_string.last().unwrap().end_connected_node
                );
            }

            // Convert geometry to the target type (MultiGeometry)
            geometries.push(geometry.into());
        }

        geometries
    }
}

impl fmt::Display for LineElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]: {} {} {}",
            if self.direction == Direction::Forward {
                "forward"
            } else {
                "reverse"
            },
            self.start_connected_node,
            self.edge_vector,
            self.end_connected_node
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(dead_code, non_camel_case_types)]
pub enum S57Attribute {
    Unknown = 0,
    /// Agency responsible for production
    AGENCY = 1,
    /// Beacon shape
    BCNSHP = 2,
    /// Building shape
    BUISHP = 3,
    /// Buoy shape
    BOYSHP = 4,
    /// Buried depth
    BURDEP = 5,
    /// Call sign
    CALSGN = 6,
    /// Category of airport/airfield
    CATAIR = 7,
    /// Category of anchorage
    CATACH = 8,
    /// Category of bridge
    CATBRG = 9,
    /// Category of built-up area
    CATBUA = 10,
    /// Category of cable
    CATCBL = 11,
    /// Category of canal
    CATCAN = 12,
    /// Category of cardinal mark
    CATCAM = 13,
    /// Category of checkpoint
    CATCHP = 14,
    /// Category of coastline
    CATCOA = 15,
    /// Category of control point
    CATCTR = 16,
    /// Category of conveyor
    CATCON = 17,
    /// Category of coverage
    CATCOV = 18,
    /// Category of crane
    CATCRN = 19,
    /// Category of dam
    CATDAM = 20,
    /// Category of distance mark
    CATDIS = 21,
    /// Category of dock
    CATDOC = 22,
    /// Category of dumping ground
    CATDPG = 23,
    /// Category of fenceline
    CATFNC = 24,
    /// Category of ferry
    CATFRY = 25,
    /// Category of fishing facility
    CATFIF = 26,
    /// Category of fog signal
    CATFOG = 27,
    /// Category of fortified structure
    CATFOR = 28,
    /// Category of gate
    CATGAT = 29,
    /// Category of ice
    CATICE = 32,
    /// Category of installation buoy
    CATINB = 33,
    /// Category of land region
    CATLND = 34,
    /// Category of landmark
    CATLMK = 35,
    /// Category of lateral mark
    CATLAM = 36,
    /// Category of light
    CATLIT = 37,
    /// Category of marine farm/culture
    CATMFA = 38,
    /// Category of military practice area
    CATMPA = 39,
    /// Category of mooring/warping facility
    CATMOR = 40,
    /// Category of obstruction
    CATOBS = 42,
    /// Category of offshore platform
    CATOFP = 43,
    /// Category of oil barrier
    CATOLB = 44,
    /// Category of pile
    CATPLE = 45,
    /// Category of pilot boarding place
    CATPIL = 46,
    /// Category of pipeline/pipe
    CATPIP = 47,
    /// Category of production area
    CATPRA = 48,
    /// Category of pylon
    CATPYL = 49,
    /// Category of quality of data
    CATQUA = 50,
    /// Category of radar station
    CATRAS = 51,
    /// Category of radar transponder beacon
    CATRTB = 52,
    /// Category of radio station
    CATROS = 53,
    /// Category of recommended track
    CATTRK = 54,
    /// Category of rescue station
    CATRSC = 55,
    /// Category of restricted area
    CATREA = 56,
    /// Category of road
    CATROD = 57,
    /// Category of runway
    CATRUN = 58,
    /// Category of sea area
    CATSEA = 59,
    /// Category of shoreline construction
    CATSLC = 60,
    /// Category of signal station, traffic
    CATSIT = 61,
    /// Category of signal station, warning
    CATSIW = 62,
    /// Category of silo/tank
    CATSIL = 63,
    /// Category of slope
    CATSLO = 64,
    /// Category of small craft facility
    CATSCF = 65,
    /// Category of special purpose mark
    CATSPM = 66,
    /// Category of Tidal Stream
    CAT_TS = 188,
    /// Category of Traffic Separation Scheme
    CATTSS = 67,
    /// Category of vegetation
    CATVEG = 68,
    /// Category of water turbulence
    CATWAT = 69,
    /// Category of weed/kelp
    CATWED = 70,
    /// Category of wreck
    CATWRK = 71,
    /// Character spacing
    SPACE = 73,
    /// Character specification
    CHARS = 74,
    /// Colour
    COLOUR = 75,
    /// Colour pattern
    COLPAT = 76,
    /// Communication channel
    COMCHA = 77,
    /// Compass size
    CSIZE = 78,
    /// Compilation date
    CPDATE = 79,
    /// Compilation scale
    CSCALE = 80,
    /// Condition
    CONDTN = 81,
    /// Conspicuous, radar
    CONRAD = 82,
    /// Conspicuous, visually
    CONVIS = 83,
    /// Current velocity
    CURVEL = 84,
    /// Date end
    DATEND = 85,
    /// Date start
    DATSTA = 86,
    /// Depth range value 1
    DRVAL1 = 87,
    /// Depth range value 2
    DRVAL2 = 88,
    /// Depth units
    DUNITS = 89,
    /// Elevation
    ELEVAT = 90,
    /// Estimated range of transmission
    ESTRNG = 91,
    /// Exposition of sounding
    EXPSOU = 93,
    /// Function
    FUNCTN = 94,
    /// Height
    HEIGHT = 95,
    /// Height/length units
    HUNITS = 96,
    /// Horizontal accuracy
    HORACC = 97,
    /// Horizontal clearance
    HORCLR = 98,
    /// Horizontal length
    HORLEN = 99,
    /// Horizontal width
    HORWID = 100,
    /// Ice factor
    ICEFAC = 101,
    /// Information
    INFORM = 102,
    /// Jurisdiction
    JRSDTN = 103,
    /// Justification - horizontal
    JUSTH = 104,
    /// Justification - vertical
    JUSTV = 105,
    /// Lifting capacity
    LIFCAP = 106,
    /// Light characteristic
    LITCHR = 107,
    /// Light visibility
    LITVIS = 108,
    /// Marks navigational - System of
    MARSYS = 109,
    /// Multiplicity of lights
    MLTYLT = 110,
    /// Nationality
    NATION = 111,
    /// Nature of construction
    NATCON = 112,
    /// Nature of surface
    NATSUR = 113,
    /// Nature of surface - qualifying terms
    NATQUA = 114,
    /// Notice to Mariners date
    NMDATE = 115,
    /// Object name
    OBJNAM = 116,
    /// Orientation
    ORIENT = 117,
    /// Periodic date end
    PEREND = 118,
    /// Periodic date start
    PERSTA = 119,
    /// Pictorial representation
    PICREP = 120,
    /// Pilot district
    PILDST = 121,
    /// Positional accuracy units
    PUNITS = 189,
    /// Producing country
    PRCTRY = 122,
    /// Product
    PRODCT = 123,
    /// Publication reference
    PUBREF = 124,
    /// Quality of sounding measurement
    QUASOU = 125,
    /// Radar wave length
    RADWAL = 126,
    /// Radius
    RADIUS = 127,
    /// Recording date
    RECDAT = 128,
    /// Recording indication
    RECIND = 129,
    /// Reference year for magnetic variation
    RYRMGV = 130,
    /// Restriction
    RESTRN = 131,
    /// Scale maximum
    SCAMAX = 132,
    /// Scale minimum
    SCAMIN = 133,
    /// Scale value one
    SCVAL1 = 134,
    /// Scale value two
    SCVAL2 = 135,
    /// Sector limit one
    SECTR1 = 136,
    /// Sector limit two
    SECTR2 = 137,
    /// Shift parameters
    SHIPAM = 138,
    /// Signal frequency
    SIGFRQ = 139,
    /// Signal generation
    SIGGEN = 140,
    /// Signal group
    SIGGRP = 141,
    /// Signal period
    SIGPER = 142,
    /// Signal sequence
    SIGSEQ = 143,
    /// Sounding accuracy
    SOUACC = 144,
    /// Sounding distance - maximum
    SDISMX = 145,
    /// Sounding distance - minimum
    SDISMN = 146,
    /// Source date
    SORDAT = 147,
    /// Source indication
    SORIND = 148,
    /// Status
    STATUS = 149,
    /// Survey date - end
    SUREND = 151,
    /// Survey date - start
    SURSTA = 152,
    /// Survey type
    SURTYP = 153,
    /// Symbol scaling factor
    SCALE = 154,
    /// Symbolization code
    SCODE = 155,
    /// Technique of sounding measurement
    TECSOU = 156,
    /// Text string
    TXSTR = 157,
    /// Textual description
    TXTDSC = 158,
    /// Tidal stream - panel values
    TS_TSP = 159,
    /// Tidal stream - time series values
    TS_TSV = 160,
    /// Tide - accuracy of water level
    T_ACWL = 161,
    /// Tide - high and low water values
    T_HWLW = 162,
    /// Tide - method of tidal prediction
    T_MTOD = 163,
    /// Tide - time and height differences
    T_THDF = 164,
    /// Tide - time series values
    T_TSVL = 166,
    /// Tide - value of harmonic constituents
    T_VAHC = 167,
    /// Tide - time interval of values
    T_TINT = 165,
    /// Time end
    TIMEND = 168,
    /// Time start
    TIMSTA = 169,
    /// Tint
    TINTS = 170,
    /// Topmark/daymark shape
    TOPSHP = 171,
    /// Traffic flow
    TRAFIC = 172,
    /// Value of annual change in magnetic variation
    VALACM = 173,
    /// Value of depth contour
    VALDCO = 174,
    /// Value of local magnetic anomaly
    VALLMA = 175,
    /// Value of magnetic variation
    VALMAG = 176,
    /// Value of maximum range
    VALMXR = 177,
    /// Value of nominal range
    VALNMR = 178,
    /// Value of sounding
    VALSOU = 179,
    /// Vertical accuracy
    VERACC = 180,
    /// Vertical clearance
    VERCLR = 181,
    /// Vertical clearance, closed
    VERCCL = 182,
    /// Vertical clearance, open
    VERCOP = 183,
    /// Vertical clearance, safe
    VERCSA = 184,
    /// Vertical datum
    VERDAT = 185,
    /// Vertical length
    VERLEN = 186,
    /// Water level effect
    WATLEV = 187,
    /// Information in national language
    NINFOM = 300,
    /// Object name in national language
    NOBJNM = 301,
    /// Pilot district in national language
    NPLDST = 302,
    /// Text string in national language
    NTXST = 303,
    /// Textual description in national language
    NTXTDS = 304,
    /// Horizontal datum
    HORDAT = 400,
    /// Positional Accuracy
    POSACC = 401,
    /// Quality of position
    QUAPOS = 402,
}

#[allow(dead_code)]
impl S57Attribute {
    pub fn from_type_code(type_code: i32) -> Self {
        match type_code {
            1 => S57Attribute::AGENCY,
            2 => S57Attribute::BCNSHP,
            3 => S57Attribute::BUISHP,
            4 => S57Attribute::BOYSHP,
            5 => S57Attribute::BURDEP,
            6 => S57Attribute::CALSGN,
            7 => S57Attribute::CATAIR,
            8 => S57Attribute::CATACH,
            9 => S57Attribute::CATBRG,
            10 => S57Attribute::CATBUA,
            11 => S57Attribute::CATCBL,
            12 => S57Attribute::CATCAN,
            13 => S57Attribute::CATCAM,
            14 => S57Attribute::CATCHP,
            15 => S57Attribute::CATCOA,
            16 => S57Attribute::CATCTR,
            17 => S57Attribute::CATCON,
            18 => S57Attribute::CATCOV,
            19 => S57Attribute::CATCRN,
            20 => S57Attribute::CATDAM,
            21 => S57Attribute::CATDIS,
            22 => S57Attribute::CATDOC,
            23 => S57Attribute::CATDPG,
            24 => S57Attribute::CATFNC,
            25 => S57Attribute::CATFRY,
            26 => S57Attribute::CATFIF,
            27 => S57Attribute::CATFOG,
            28 => S57Attribute::CATFOR,
            29 => S57Attribute::CATGAT,
            32 => S57Attribute::CATICE,
            33 => S57Attribute::CATINB,
            34 => S57Attribute::CATLND,
            35 => S57Attribute::CATLMK,
            36 => S57Attribute::CATLAM,
            37 => S57Attribute::CATLIT,
            38 => S57Attribute::CATMFA,
            39 => S57Attribute::CATMPA,
            40 => S57Attribute::CATMOR,
            42 => S57Attribute::CATOBS,
            43 => S57Attribute::CATOFP,
            44 => S57Attribute::CATOLB,
            45 => S57Attribute::CATPLE,
            46 => S57Attribute::CATPIL,
            47 => S57Attribute::CATPIP,
            48 => S57Attribute::CATPRA,
            49 => S57Attribute::CATPYL,
            50 => S57Attribute::CATQUA,
            51 => S57Attribute::CATRAS,
            52 => S57Attribute::CATRTB,
            53 => S57Attribute::CATROS,
            54 => S57Attribute::CATTRK,
            55 => S57Attribute::CATRSC,
            56 => S57Attribute::CATREA,
            57 => S57Attribute::CATROD,
            58 => S57Attribute::CATRUN,
            59 => S57Attribute::CATSEA,
            60 => S57Attribute::CATSLC,
            61 => S57Attribute::CATSIT,
            62 => S57Attribute::CATSIW,
            63 => S57Attribute::CATSIL,
            64 => S57Attribute::CATSLO,
            65 => S57Attribute::CATSCF,
            66 => S57Attribute::CATSPM,
            67 => S57Attribute::CATTSS,
            68 => S57Attribute::CATVEG,
            69 => S57Attribute::CATWAT,
            70 => S57Attribute::CATWED,
            71 => S57Attribute::CATWRK,
            73 => S57Attribute::SPACE,
            74 => S57Attribute::CHARS,
            75 => S57Attribute::COLOUR,
            76 => S57Attribute::COLPAT,
            77 => S57Attribute::COMCHA,
            78 => S57Attribute::CSIZE,
            79 => S57Attribute::CPDATE,
            80 => S57Attribute::CSCALE,
            81 => S57Attribute::CONDTN,
            82 => S57Attribute::CONRAD,
            83 => S57Attribute::CONVIS,
            84 => S57Attribute::CURVEL,
            85 => S57Attribute::DATEND,
            86 => S57Attribute::DATSTA,
            87 => S57Attribute::DRVAL1,
            88 => S57Attribute::DRVAL2,
            89 => S57Attribute::DUNITS,
            90 => S57Attribute::ELEVAT,
            91 => S57Attribute::ESTRNG,
            93 => S57Attribute::EXPSOU,
            94 => S57Attribute::FUNCTN,
            95 => S57Attribute::HEIGHT,
            96 => S57Attribute::HUNITS,
            97 => S57Attribute::HORACC,
            98 => S57Attribute::HORCLR,
            99 => S57Attribute::HORLEN,
            100 => S57Attribute::HORWID,
            101 => S57Attribute::ICEFAC,
            102 => S57Attribute::INFORM,
            103 => S57Attribute::JRSDTN,
            104 => S57Attribute::JUSTH,
            105 => S57Attribute::JUSTV,
            106 => S57Attribute::LIFCAP,
            107 => S57Attribute::LITCHR,
            108 => S57Attribute::LITVIS,
            109 => S57Attribute::MARSYS,
            110 => S57Attribute::MLTYLT,
            111 => S57Attribute::NATION,
            112 => S57Attribute::NATCON,
            113 => S57Attribute::NATSUR,
            114 => S57Attribute::NATQUA,
            115 => S57Attribute::NMDATE,
            116 => S57Attribute::OBJNAM,
            117 => S57Attribute::ORIENT,
            118 => S57Attribute::PEREND,
            119 => S57Attribute::PERSTA,
            120 => S57Attribute::PICREP,
            121 => S57Attribute::PILDST,
            122 => S57Attribute::PRCTRY,
            123 => S57Attribute::PRODCT,
            124 => S57Attribute::PUBREF,
            125 => S57Attribute::QUASOU,
            126 => S57Attribute::RADWAL,
            127 => S57Attribute::RADIUS,
            128 => S57Attribute::RECDAT,
            129 => S57Attribute::RECIND,
            130 => S57Attribute::RYRMGV,
            131 => S57Attribute::RESTRN,
            132 => S57Attribute::SCAMAX,
            133 => S57Attribute::SCAMIN,
            134 => S57Attribute::SCVAL1,
            135 => S57Attribute::SCVAL2,
            136 => S57Attribute::SECTR1,
            137 => S57Attribute::SECTR2,
            138 => S57Attribute::SHIPAM,
            139 => S57Attribute::SIGFRQ,
            140 => S57Attribute::SIGGEN,
            141 => S57Attribute::SIGGRP,
            142 => S57Attribute::SIGPER,
            143 => S57Attribute::SIGSEQ,
            144 => S57Attribute::SOUACC,
            145 => S57Attribute::SDISMX,
            146 => S57Attribute::SDISMN,
            147 => S57Attribute::SORDAT,
            148 => S57Attribute::SORIND,
            149 => S57Attribute::STATUS,
            151 => S57Attribute::SUREND,
            152 => S57Attribute::SURSTA,
            153 => S57Attribute::SURTYP,
            154 => S57Attribute::SCALE,
            155 => S57Attribute::SCODE,
            156 => S57Attribute::TECSOU,
            157 => S57Attribute::TXSTR,
            158 => S57Attribute::TXTDSC,
            159 => S57Attribute::TS_TSP,
            160 => S57Attribute::TS_TSV,
            161 => S57Attribute::T_ACWL,
            162 => S57Attribute::T_HWLW,
            163 => S57Attribute::T_MTOD,
            164 => S57Attribute::T_THDF,
            165 => S57Attribute::T_TINT,
            166 => S57Attribute::T_TSVL,
            167 => S57Attribute::T_VAHC,
            168 => S57Attribute::TIMEND,
            169 => S57Attribute::TIMSTA,
            170 => S57Attribute::TINTS,
            171 => S57Attribute::TOPSHP,
            172 => S57Attribute::TRAFIC,
            173 => S57Attribute::VALACM,
            174 => S57Attribute::VALDCO,
            175 => S57Attribute::VALLMA,
            176 => S57Attribute::VALMAG,
            177 => S57Attribute::VALMXR,
            178 => S57Attribute::VALNMR,
            179 => S57Attribute::VALSOU,
            180 => S57Attribute::VERACC,
            181 => S57Attribute::VERCLR,
            182 => S57Attribute::VERCCL,
            183 => S57Attribute::VERCOP,
            184 => S57Attribute::VERCSA,
            185 => S57Attribute::VERDAT,
            186 => S57Attribute::VERLEN,
            187 => S57Attribute::WATLEV,
            188 => S57Attribute::CAT_TS,
            189 => S57Attribute::PUNITS,
            300 => S57Attribute::NINFOM,
            301 => S57Attribute::NOBJNM,
            302 => S57Attribute::NPLDST,
            303 => S57Attribute::NTXST,
            304 => S57Attribute::NTXTDS,
            400 => S57Attribute::HORDAT,
            401 => S57Attribute::POSACC,
            402 => S57Attribute::QUAPOS,
            _ => S57Attribute::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code, non_camel_case_types)]
pub enum S57Type {
    Unknown = 0,
    ADMARE = 1,   // Administration Area (Named)
    AIRARE = 2,   // Airport/airfield
    ACHBRT = 3,   // Anchor berth
    ACHARE = 4,   // Anchorage area
    BCNCAR = 5,   // Beacon, cardinal
    BCNISD = 6,   // Beacon, isolated danger
    BCNLAT = 7,   // Beacon, lateral
    BCNSAW = 8,   // Beacon, safe water
    BCNSPP = 9,   // Beacon, special purpose/general
    BERTHS = 10,  // Berth
    BRIDGE = 11,  // Bridge
    BUISGL = 12,  // Building, single
    BUAARE = 13,  // Built-up area
    BOYCAR = 14,  // Buoy, cardinal
    BOYINB = 15,  // Buoy, installation
    BOYISD = 16,  // Buoy, isolated danger
    BOYLAT = 17,  // Buoy, lateral
    BOYSAW = 18,  // Buoy, safe water
    BOYSPP = 19,  // Buoy, special purpose/general
    CBLARE = 20,  // Cable area
    CBLOHD = 21,  // Cable, overhead
    CBLSUB = 22,  // Cable, submarine
    CANALS = 23,  // Canal
    CANBNK = 24,  // Canal bank
    CTSARE = 25,  // Cargo transhipment area
    CAUSWY = 26,  // Causeway
    CTNARE = 27,  // Caution area
    CHKPNT = 28,  // Checkpoint
    CGUSTA = 29,  // Coastguard station
    COALNE = 30,  // Coastline
    CONZNE = 31,  // Contiguous zone
    COSARE = 32,  // Continental shelf area
    CTRPNT = 33,  // Control point
    CONVYR = 34,  // Conveyor
    CRANES = 35,  // Crane
    CURENT = 36,  // Current - non-gravitational1
    CUSZNE = 37,  // Custom zone
    DAMCON = 38,  // Dam
    DAYMAR = 39,  // Daymark
    DWRTCL = 40,  // Deep water route centerline
    DWRTPT = 41,  // Deep water route part
    DEPARE = 42,  // Depth area
    DEPCNT = 43,  // Depth contour
    DISMAR = 44,  // Distance mark
    DOCARE = 45,  // Dock area
    DRGARE = 46,  // Dredged area
    DRYDOC = 47,  // Dry dock
    DMPGRD = 48,  // Dumping ground
    DYKCON = 49,  // Dyke
    EXEZNE = 50,  // Exclusive economic zone
    FAIRWY = 51,  // Fairway
    FNCLNE = 52,  // Fence/wall
    FERYRT = 53,  // Ferry route
    FSHZNE = 54,  // Fishery zone
    FSHFAC = 55,  // Fishing facility
    FSHGRD = 56,  // Fishing ground
    FLODOC = 57,  // Floating dock
    FOGSIG = 58,  // Fog signal
    FORSTC = 59,  // Fortified structure
    FRPARE = 60,  // Free port area
    GATCON = 61,  // Gate
    GRIDRN = 62,  // Gridiron
    HRBARE = 63,  // Harbour area (administrative)
    HRBFAC = 64,  // Harbour facility
    HULKES = 65,  // Hulk
    ICEARE = 66,  // Ice area
    ICNARE = 67,  // Incineration area
    ISTZNE = 68,  // Inshore traffic zone
    LAKARE = 69,  // Lake
    LAKSHR = 70,  // Lake shore
    LNDARE = 71,  // Land area
    LNDELV = 72,  // Land elevation
    LNDRGN = 73,  // Land region
    LNDMRK = 74,  // Landmark
    LIGHTS = 75,  // Light
    LITFLT = 76,  // Light float
    LITVES = 77,  // Light vessel
    LOCMAG = 78,  // Local magnetic anomaly
    LOKBSN = 79,  // Lock basin
    LOGPON = 80,  // Log pond
    MAGVAR = 81,  // Magnetic variation
    MARCUL = 82,  // Marine farm/culture
    MIPARE = 83,  // Military practice area
    MORFAC = 84,  // Mooring/Warping facility
    NAVLNE = 85,  // Navigation line
    OBSTRN = 86,  // Obstruction
    OFSPLF = 87,  // Offshore platform
    OSPARE = 88,  // Offshore production area
    OILBAR = 89,  // Oil barrier
    PILPNT = 90,  // Pile
    PILBOP = 91,  // Pilot boarding place
    PIPARE = 92,  // Pipeline area
    PIPOHD = 93,  // Pipeline, overhead
    PIPSOL = 94,  // Pipeline, submarine/on land
    PONTON = 95,  // Pontoon
    PRCARE = 96,  // Precautionary area
    PRDARE = 97,  // Production/storage area
    PYLONS = 98,  // Pylon/bridge support
    RADLNE = 99,  // Radar line
    RADRNG = 100, // Radar range
    RADRFL = 101, // Radar reflector
    RADSTA = 102, // Radar station
    RTPBCN = 103, // Radar transponder beacon
    RDOCAL = 104, // Radio calling-in point
    RDOSTA = 105, // Radio station
    RAILWY = 106, // Railway
    RAPIDS = 107, // Rapids
    RCRTCL = 108, // Recommended route centerline
    RECTRC = 109, // Recommended track
    RCTLPT = 110, // Recommended traffic lane part
    RSCSTA = 111, // Rescue station
    RESARE = 112, // Restricted area
    RETRFL = 113, // Retro-reflector
    RIVERS = 114, // River
    RIVBNK = 115, // River bank
    ROADWY = 116, // Road
    RUNWAY = 117, // Runway
    SNDWAV = 118, // Sand waves
    SEAARE = 119, // Sea area/named water area
    SPLARE = 120, // Sea-plane landing area
    SBDARE = 121, // Seabed area
    SLCONS = 122, // Shoreline construction
    SISTAT = 123, // Signal station, traffic
    SISTAW = 124, // Signal station, warning
    SILTNK = 125, // Silo/tank
    SLOTOP = 126, // Slope topline
    SLOGRD = 127, // Sloping ground
    SMCFAC = 128, // Small craft facility
    SOUNDG = 129, // Sounding
    SPRING = 130, // Spring
    SQUARE = 131, // Square
    STSLNE = 132, // Straight territorial sea baseline
    SUBTLN = 133, // Submarine transit lane
    SWPARE = 134, // Swept Area
    TESARE = 135, // Territorial sea area
    TS_PRH = 136, // Tidal stream - harmonic prediction
    TS_FEB = 160, // Tidal stream - flood / ebb
    TS_PNH = 137, // Tidal stream - non-harmonic prediction
    TS_PAD = 138, // Tidal stream panel data
    TS_TIS = 139, // Tidal stream - time series
    T_HMON = 140, // Tide - harmonic prediction
    T_NHMN = 141, // Tide - non-harmonic prediction
    T_TIMS = 142, // Tide - time series
    TIDEWY = 143, // Tideway
    TOPMAR = 144, // Topmark
    TSELNE = 145, // Traffic separation line
    TSSBND = 146, // Traffic separation scheme boundary
    TSSCRS = 147, // Traffic separation scheme crossing
    TSSLPT = 148, // Traffic separation scheme lane part
    TSSRON = 149, // Traffic separation scheme roundabout
    TSEZNE = 150, // Traffic separation zone
    TUNNEL = 151, // Tunnel
    TWRTPT = 152, // Two-way route part
    UWTROC = 153, // Underwater/awash rock
    UNSARE = 154, // Unsurveyed area
    VEGATN = 155, // Vegetation
    WATTUR = 156, // Water turbulence
    WATFAL = 157, // Waterfall
    WEDKLP = 158, // Weed/Kelp
    WRECKS = 159, // Wreck

    M_ACCY = 300, // Accuracy of data
    M_CSCL = 301, // Compilation scale of data
    M_COVR = 302, // Coverage
    M_HDAT = 303, // Horizontal datum of data
    M_HOPA = 304, // Horizontal datum shift parameters
    M_NPUB = 305, // Nautical publication information
    M_NSYS = 306, // Navigational system of marks
    M_PROD = 307, // Production information
    M_QUAL = 308, // Quality of data
    M_SDAT = 309, // Sounding datum
    M_SREL = 310, // Survey reliability
    M_UNIT = 311, // Units of measurement of data
    M_VDAT = 312, // Vertical datum of data

    C_AGGR = 400, // Aggregation
    C_ASSO = 401, // Association
    C_STAC = 402, // Stacked on/stacked under

    AREAS = 500, // Cartographic area
    LINES = 501, // Cartographic line
    CSYMB = 502, // Cartographic symbol
    COMPS = 503, // Compass
    TEXTS = 504, // Text
}
#[allow(dead_code)]
impl S57Type {
    pub fn from_type_code(type_code: i32) -> S57Type {
        match type_code {
            0 => S57Type::Unknown,
            1 => S57Type::ADMARE,
            2 => S57Type::AIRARE,
            3 => S57Type::ACHBRT,
            4 => S57Type::ACHARE,
            5 => S57Type::BCNCAR,
            6 => S57Type::BCNISD,
            7 => S57Type::BCNLAT,
            8 => S57Type::BCNSAW,
            9 => S57Type::BCNSPP,
            10 => S57Type::BERTHS,
            11 => S57Type::BRIDGE,
            12 => S57Type::BUISGL,
            13 => S57Type::BUAARE,
            14 => S57Type::BOYCAR,
            15 => S57Type::BOYINB,
            16 => S57Type::BOYISD,
            17 => S57Type::BOYLAT,
            18 => S57Type::BOYSAW,
            19 => S57Type::BOYSPP,
            20 => S57Type::CBLARE,
            21 => S57Type::CBLOHD,
            22 => S57Type::CBLSUB,
            23 => S57Type::CANALS,
            24 => S57Type::CANBNK,
            25 => S57Type::CTSARE,
            26 => S57Type::CAUSWY,
            27 => S57Type::CTNARE,
            28 => S57Type::CHKPNT,
            29 => S57Type::CGUSTA,
            30 => S57Type::COALNE,
            31 => S57Type::CONZNE,
            32 => S57Type::COSARE,
            33 => S57Type::CTRPNT,
            34 => S57Type::CONVYR,
            35 => S57Type::CRANES,
            36 => S57Type::CURENT,
            37 => S57Type::CUSZNE,
            38 => S57Type::DAMCON,
            39 => S57Type::DAYMAR,
            40 => S57Type::DWRTCL,
            41 => S57Type::DWRTPT,
            42 => S57Type::DEPARE,
            43 => S57Type::DEPCNT,
            44 => S57Type::DISMAR,
            45 => S57Type::DOCARE,
            46 => S57Type::DRGARE,
            47 => S57Type::DRYDOC,
            48 => S57Type::DMPGRD,
            49 => S57Type::DYKCON,
            50 => S57Type::EXEZNE,
            51 => S57Type::FAIRWY,
            52 => S57Type::FNCLNE,
            53 => S57Type::FERYRT,
            54 => S57Type::FSHZNE,
            55 => S57Type::FSHFAC,
            56 => S57Type::FSHGRD,
            57 => S57Type::FLODOC,
            58 => S57Type::FOGSIG,
            59 => S57Type::FORSTC,
            60 => S57Type::FRPARE,
            61 => S57Type::GATCON,
            62 => S57Type::GRIDRN,
            63 => S57Type::HRBARE,
            64 => S57Type::HRBFAC,
            65 => S57Type::HULKES,
            66 => S57Type::ICEARE,
            67 => S57Type::ICNARE,
            68 => S57Type::ISTZNE,
            69 => S57Type::LAKARE,
            70 => S57Type::LAKSHR,
            71 => S57Type::LNDARE,
            72 => S57Type::LNDELV,
            73 => S57Type::LNDRGN,
            74 => S57Type::LNDMRK,
            75 => S57Type::LIGHTS,
            76 => S57Type::LITFLT,
            77 => S57Type::LITVES,
            78 => S57Type::LOCMAG,
            79 => S57Type::LOKBSN,
            80 => S57Type::LOGPON,
            81 => S57Type::MAGVAR,
            82 => S57Type::MARCUL,
            83 => S57Type::MIPARE,
            84 => S57Type::MORFAC,
            85 => S57Type::NAVLNE,
            86 => S57Type::OBSTRN,
            87 => S57Type::OFSPLF,
            88 => S57Type::OSPARE,
            89 => S57Type::OILBAR,
            90 => S57Type::PILPNT,
            91 => S57Type::PILBOP,
            92 => S57Type::PIPARE,
            93 => S57Type::PIPOHD,
            94 => S57Type::PIPSOL,
            95 => S57Type::PONTON,
            96 => S57Type::PRCARE,
            97 => S57Type::PRDARE,
            98 => S57Type::PYLONS,
            99 => S57Type::RADLNE,
            100 => S57Type::RADRNG,
            101 => S57Type::RADRFL,
            102 => S57Type::RADSTA,
            103 => S57Type::RTPBCN,
            104 => S57Type::RDOCAL,
            105 => S57Type::RDOSTA,
            106 => S57Type::RAILWY,
            107 => S57Type::RAPIDS,
            108 => S57Type::RCRTCL,
            109 => S57Type::RECTRC,
            110 => S57Type::RCTLPT,
            111 => S57Type::RSCSTA,
            112 => S57Type::RESARE,
            113 => S57Type::RETRFL,
            114 => S57Type::RIVERS,
            115 => S57Type::RIVBNK,
            116 => S57Type::ROADWY,
            117 => S57Type::RUNWAY,
            118 => S57Type::SNDWAV,
            119 => S57Type::SEAARE,
            120 => S57Type::SPLARE,
            121 => S57Type::SBDARE,
            122 => S57Type::SLCONS,
            123 => S57Type::SISTAT,
            124 => S57Type::SISTAW,
            125 => S57Type::SILTNK,
            126 => S57Type::SLOTOP,
            127 => S57Type::SLOGRD,
            128 => S57Type::SMCFAC,
            129 => S57Type::SOUNDG,
            130 => S57Type::SPRING,
            131 => S57Type::SQUARE,
            132 => S57Type::STSLNE,
            133 => S57Type::SUBTLN,
            134 => S57Type::SWPARE,
            135 => S57Type::TESARE,
            136 => S57Type::TS_PRH,
            160 => S57Type::TS_FEB,
            137 => S57Type::TS_PNH,
            138 => S57Type::TS_PAD,
            139 => S57Type::TS_TIS,
            140 => S57Type::T_HMON,
            141 => S57Type::T_NHMN,
            142 => S57Type::T_TIMS,
            143 => S57Type::TIDEWY,
            144 => S57Type::TOPMAR,
            145 => S57Type::TSELNE,
            146 => S57Type::TSSBND,
            147 => S57Type::TSSCRS,
            148 => S57Type::TSSLPT,
            149 => S57Type::TSSRON,
            150 => S57Type::TSEZNE,
            151 => S57Type::TUNNEL,
            152 => S57Type::TWRTPT,
            153 => S57Type::UWTROC,
            154 => S57Type::UNSARE,
            155 => S57Type::VEGATN,
            156 => S57Type::WATTUR,
            157 => S57Type::WATFAL,
            158 => S57Type::WEDKLP,
            159 => S57Type::WRECKS,
            300 => S57Type::M_ACCY,
            301 => S57Type::M_CSCL,
            302 => S57Type::M_COVR,
            303 => S57Type::M_HDAT,
            304 => S57Type::M_HOPA,
            305 => S57Type::M_NPUB,
            306 => S57Type::M_NSYS,
            307 => S57Type::M_PROD,
            308 => S57Type::M_QUAL,
            309 => S57Type::M_SDAT,
            310 => S57Type::M_SREL,
            311 => S57Type::M_UNIT,
            312 => S57Type::M_VDAT,
            400 => S57Type::C_AGGR,
            401 => S57Type::C_ASSO,
            402 => S57Type::C_STAC,
            500 => S57Type::AREAS,
            501 => S57Type::LINES,
            502 => S57Type::CSYMB,
            503 => S57Type::COMPS,
            504 => S57Type::TEXTS,
            _ => S57Type::Unknown,
        }
    }
}
