#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Quadrant {
    Platforms,
    Languages,
    Tools,
    Techniques,
}

impl Quadrant {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Platforms => "platforms",
            Self::Languages => "languages",
            Self::Tools => "tools",
            Self::Techniques => "techniques",
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Platforms),
            1 => Some(Self::Languages),
            2 => Some(Self::Tools),
            3 => Some(Self::Techniques),
            _ => None,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "platforms" => Some(Self::Platforms),
            "languages" => Some(Self::Languages),
            "tools" => Some(Self::Tools),
            "techniques" => Some(Self::Techniques),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Platforms => "Platforms",
            Self::Languages => "Languages",
            Self::Tools => "Tools",
            Self::Techniques => "Techniques",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Ring {
    Hold,
    Assess,
    Trial,
    Adopt,
}

impl Ring {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hold => "hold",
            Self::Assess => "assess",
            Self::Trial => "trial",
            Self::Adopt => "adopt",
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Hold),
            1 => Some(Self::Assess),
            2 => Some(Self::Trial),
            3 => Some(Self::Adopt),
            _ => None,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "hold" => Some(Self::Hold),
            "assess" => Some(Self::Assess),
            "trial" => Some(Self::Trial),
            "adopt" => Some(Self::Adopt),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hold => "Hold",
            Self::Assess => "Assess",
            Self::Trial => "Trial",
            Self::Adopt => "Adopt",
        }
    }
}
