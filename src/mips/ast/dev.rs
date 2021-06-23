pub struct DevRaw {
    index: usize,
}

pub enum Dev {
    Dev(DevRaw),
    Indir {
        indirections: usize,
        reg: Reg,
    },
    Alias {
        name: String,
        dev: Dev,
    }
}

impl<'i> MipsNode<'i, Rule, MipsParser> for Dev {
    type Output = Self;

    const RULE: Rule = Rule::dev;

    fn try_from_pair(mips: &mut Mips, pair: Pair<Rule>) -> MipsResult<Self> {
        match pair.as_rule() {
            Rule::dev => {
                let s = pair.as_str();
                let indirections = s.bytes().filter(|b| *b == b'r').count();
                let index = pair.only_inner()?.as_str().parse()?;
                if indirections == 0 {
                    Ok(Dev::Dev(DevRaw { index }))
                } else {
                    // let reg = 
                    // Ok(Dev::Indir(
                }
            },
            _ => panic!(),
        }
    }
}
