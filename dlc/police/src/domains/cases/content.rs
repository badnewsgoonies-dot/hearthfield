use crate::shared::NpcId;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WitnessInterview {
    pub witness_id: &'static str,
    pub lines: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CaseFlavorText {
    pub opening: &'static str,
    pub mid_case_update: &'static str,
    pub resolution: &'static str,
    pub witness_interviews: &'static [WitnessInterview],
}

macro_rules! witness {
    ($witness_id:expr, [$($line:expr),+ $(,)?]) => {
        WitnessInterview {
            witness_id: $witness_id,
            lines: &[$($line),+],
        }
    };
}

const PATROL_001_WITNESSES: &[WitnessInterview] = &[witness!(
    "rita_gomez",
    [
        "Marcus kept circling the register like he was trying to memorize who was watching him.",
        "He bought coffee, paid exact change, then drifted back when Mrs. Hale ducked into the stockroom.",
        "If you ask me, he didn't plan big. He just saw a drawer open and a bad idea got loud.",
    ]
)];
const PATROL_002_WITNESSES: &[WitnessInterview] = &[witness!(
    "father_brennan",
    [
        "The paint was still tacky at dawn, which means whoever did it worked after the last bus stopped running.",
        "The children recognized the symbol, but not from church. They said it has been showing up near the rail line.",
        "Vandalism is noise with a motive. Find out who wanted the park to feel claimed.",
    ]
)];
const PATROL_003_WITNESSES: &[WitnessInterview] = &[witness!(
    "rita_gomez",
    [
        "That apartment was rattling my pie plates from two blocks over.",
        "It sounded less like a party and more like two people trying to win an argument with speakers.",
        "When the music cut out, someone yelled, 'Tell him the envelope is gone.' That's the part I'd write down.",
    ]
)];
const PATROL_004_WITNESSES: &[WitnessInterview] = &[witness!(
    "father_brennan",
    [
        "The dog didn't run scared. It ran after something, ears up and tail level.",
        "I found paw prints by the park fence and one child's mitten caught on the wire.",
        "Whatever pulled the animal away smelled familiar enough that it didn't hesitate.",
    ]
)];
const PATROL_005_WITNESSES: &[WitnessInterview] = &[witness!(
    "rita_gomez",
    [
        "The clerk shouted before the kid even hit the door, so this wasn't their first dance.",
        "Whoever lifted the watches handed the box off to someone in a gray coat outside.",
        "You don't shoplift in broad daylight unless you think the handoff matters more than the merchandise.",
    ]
)];
const PATROL_006_WITNESSES: &[WitnessInterview] = &[witness!(
    "sgt_murphy",
    [
        "The car wasn't chosen at random. Whoever hit it knew which lot cameras were still busted.",
        "They pried the passenger side first, then got impatient and snapped the lock clean.",
        "That kind of clumsy hurry usually means the thief expected to find one specific item inside.",
    ]
)];
const PATROL_007_WITNESSES: &[WitnessInterview] = &[witness!(
    "nadia_park",
    [
        "Those tags are too consistent to be kids showing off. Same pressure, same slant, same exit route.",
        "I watched a van idle nearby every time a new wall went up, but the plates were mud-smeared on purpose.",
        "Whoever is painting wants the district to feel owned before anyone realizes it's a warning.",
    ]
)];
const PATROL_008_WITNESSES: &[WitnessInterview] = &[witness!(
    "officer_chen",
    [
        "The fence tear is low and clean. Adult, athletic, carrying something light.",
        "Boot tread heads toward the loading yard, then doubles back when the train horn hits.",
        "Trespassers usually scavenge. This one moved like they were meeting someone on schedule.",
    ]
)];
const DETECTIVE_001_WITNESSES: &[WitnessInterview] = &[witness!(
    "father_brennan",
    [
        "The homeowners still had dinner plates in the sink, so the burglar learned their routine, not their valuables.",
        "A neighbor mentioned a man pretending to hand out church flyers two days before the break-in.",
        "People casing houses borrow harmless costumes. They count on us dismissing the polite version of danger.",
    ]
)];
const DETECTIVE_002_WITNESSES: &[WitnessInterview] = &[witness!(
    "dr_okafor",
    [
        "The bruising on the victim's jaw came first; the ribs happened after they hit the ground.",
        "Your attacker is right-handed and used enough force to suggest confidence, not panic.",
        "Tell your witnesses to stop saying it was mutual unless they can explain the angle of impact.",
    ]
)];
const DETECTIVE_003_WITNESSES: &[WitnessInterview] = &[witness!(
    "nadia_park",
    [
        "Fraud this tidy usually starts with one person testing how much shame the victim can absorb in silence.",
        "The forged signatures are competent, but the phone routing is lazy. Someone reused a favor chain.",
        "If the bank is nervous, it isn't because of the money. It's because the wrong ledger might name names.",
    ]
)];
const DETECTIVE_004_WITNESSES: &[WitnessInterview] = &[witness!(
    "nadia_park",
    [
        "Neighbors saw the missing woman leave twice that week: once angry, once careful.",
        "The careful trip matters. She carried a bag with no overnight weight and checked every parked car on the block.",
        "People who mean to disappear don't look over their shoulder that often. People being managed do.",
    ]
)];
const DETECTIVE_005_WITNESSES: &[WitnessInterview] = &[witness!(
    "ghost_tipster",
    [
        "The fire climbed too fast for bad wiring. Somebody fed it where the wind would do the rest.",
        "Check who moved inventory the night before the blaze and who suddenly remembered an alibi.",
        "Ash lies. Shipping ledgers don't.",
    ]
)];
const DETECTIVE_006_WITNESSES: &[WitnessInterview] = &[witness!(
    "lucia_vega",
    [
        "The same names keep appearing in possession reports, but never with enough product to justify the heat around them.",
        "That's not street luck. That's a distribution ladder using disposable arrests as camouflage.",
        "If you keep chasing cuffs instead of couriers, you'll make their paperwork cleaner and nothing else.",
    ]
)];
const DETECTIVE_007_WITNESSES: &[WitnessInterview] = &[witness!(
    "dr_okafor",
    [
        "Paint transfer on the victim's sleeve came from a truck panel, not a sedan.",
        "The blood spatter suggests the driver hesitated after impact, then accelerated again.",
        "A mechanic can hide dents. They can't hide panic baked into a repair timeline.",
    ]
)];
const DETECTIVE_008_WITNESSES: &[WitnessInterview] = &[witness!(
    "mayor_aldridge",
    [
        "I receive unpleasant letters every week. The difference here is precision.",
        "The writer knew which committee vote I delayed and which donor stopped calling afterward.",
        "Find the person who profits from embarrassment with legal cover, and you'll find your blackmailer.",
    ]
)];
const SERGEANT_001_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "dr_okafor",
        [
            "The victim fought back hard enough to skin two knuckles on the attacker's clothing or jewelry.",
            "Time of death puts them alive during the first round of emergency calls, which narrows your window nicely.",
            "This wasn't efficient. It was personal with a stopwatch running.",
        ]
    ),
    witness!(
        "ghost_tipster",
        [
            "Your dead man was seen arguing with someone who never pays for drinks in cash.",
            "Ask who cleared a side street right before midnight and why patrol got sent the long way around.",
            "Somebody wanted witnesses scattered before the body cooled.",
        ]
    ),
];
const SERGEANT_002_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "father_brennan",
        [
            "The child's mother keeps blaming herself for a routine that had never failed before.",
            "A blue sedan idled near the curb during dismissal three days in a row before the abduction.",
            "Predators rehearse normal until the neighborhood stops seeing them.",
        ]
    ),
    witness!(
        "nadia_park",
        [
            "A highway camera glitched at exactly the moment that sedan should have crossed frame.",
            "Glitches that neat usually come with a phone call and someone who owes a favor.",
            "Treat the broken footage like a message, not a setback.",
        ]
    ),
];
const SERGEANT_003_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "rita_gomez",
        [
            "The same crew has been buying burner coffee on alternating corners so nobody clocks the pattern.",
            "One of them always asks which security shutters still jam in bad weather.",
            "They aren't fencing random loot. They're timing neighborhoods like delivery routes.",
        ]
    ),
    witness!(
        "ghost_tipster",
        [
            "The ring stores hot goods in places too boring to search first.",
            "Check invoices, not hideouts. Organized thieves love paperwork when it belongs to someone else.",
            "And keep Marcus talking. He's scared of the ring, which means he knows its edges.",
        ]
    ),
];
const SERGEANT_004_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "lucia_vega",
        [
            "When good officers start writing the same bad report language, someone is coaching them.",
            "I have clients who suddenly remember badge numbers only after their charges disappear.",
            "If you keep this case clean, the guilty won't beat it on law. They'll beat it on your colleagues' panic.",
        ]
    ),
    witness!(
        "nadia_park",
        [
            "A source fed me bank records tied to a shell company that bills the department for equipment no one has seen.",
            "The paper trail bends back toward precinct parking access and a friendly judge's clerk.",
            "Corruption always pretends to be administrative until the money gets nervous.",
        ]
    ),
];
const SERGEANT_005_WITNESSES: &[WitnessInterview] = &[witness!(
    "dr_okafor",
    [
        "The old biological sample degraded, but not beyond rescue. Whoever logged it originally mislabeled the storage box.",
        "That error could be negligence or intent. The distinction is your whole reopened case.",
        "Modern testing doesn't care how long a lie has been sitting in evidence.",
    ]
)];
const SERGEANT_006_WITNESSES: &[WitnessInterview] = &[witness!(
    "father_brennan",
    [
        "The damage keeps skipping houses with floodlights and landing near homes under financial strain.",
        "That makes it pattern, not tantrum. Someone is selecting targets that won't fight back publicly.",
        "Serial vandals still tell stories. Their language is just made of broken things.",
    ]
)];
const LIEUTENANT_001_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "dr_okafor",
        [
            "Across the scenes, the injuries escalate in confidence, not chaos.",
            "Your killer is learning from each success and correcting mistakes between bodies.",
            "If the next scene happens before you're ready, it won't be because the evidence failed you.",
        ]
    ),
    witness!(
        "nadia_park",
        [
            "The city desk thinks these murders are isolated because the victims don't match socially.",
            "But the gaps between them do. Same cooling-off period, same media silence, same road access.",
            "The pattern is hiding in logistics, not personality.",
        ]
    ),
];
const LIEUTENANT_002_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "mayor_aldridge",
        [
            "You are asking questions that make campaign staff, developers, and donors all call me in the same hour.",
            "That only happens when separate people discover they share the same vulnerability.",
            "If there is a conspiracy here, it survives because each participant thinks someone else owns the whole map.",
        ]
    ),
    witness!(
        "ghost_tipster",
        [
            "You won't crack this by chasing the loudest name in the room.",
            "Find the quiet accountant, the scheduler, the clerk with a perfect memory and a bad mortgage.",
            "Cities rot through bookkeeping before they rot in public.",
        ]
    ),
];
const LIEUTENANT_003_WITNESSES: &[WitnessInterview] = &[
    witness!(
        "captain_torres",
        [
            "Every institution you've leaned on is about to ask what happens if you're right.",
            "This is the kind of case that turns careers into footnotes if the chain of proof breaks once.",
            "Do the work so cleanly that nobody gets to call your instincts the evidence.",
        ]
    ),
    witness!(
        "det_vasquez",
        [
            "We've both been circling this shape for months; now it finally has a face.",
            "Don't rush because the answer is close. Close cases are where cops start editing reality to feel relief.",
            "We finish it together or not at all.",
        ]
    ),
];

const PATROL_001: CaseFlavorText = CaseFlavorText {
    opening: "Captain Torres wants the store theft cleared before the owner starts naming every poor kid on the block. The cash drawer was pinched during a narrow blind spot, and the first witness already has a favorite suspect.",
    mid_case_update: "The simple shop theft is looking more opportunistic than organized. The timing, register access, and witness rhythm all point to someone who knew exactly how long the clerk would be away.",
    resolution: "You close the store theft with a clean chain of proof and stop a petty grab from turning into neighborhood folklore about who the precinct protects.",
    witness_interviews: PATROL_001_WITNESSES,
};
const PATROL_002: CaseFlavorText = CaseFlavorText {
    opening: "Parks crews want the vandalism documented before weather and joggers erase the trail. The damage carries more intent than random spray paint, and the neighborhood wants to know whether this is provocation or posturing.",
    mid_case_update: "Fresh markings and movement around the park suggest the vandal treated the scene like a message board, not a dare. The pattern points outward toward the rail line and industrial drift.",
    resolution: "The park opens without rumor swallowing the facts, and the case reads as a deliberate act solved through patient scene work rather than civic panic.",
    witness_interviews: PATROL_002_WITNESSES,
};
const PATROL_003: CaseFlavorText = CaseFlavorText {
    opening: "The noise complaint sounds routine until dispatch notes how many neighbors called in under separate names. Whatever happened in that apartment rattled more than windows.",
    mid_case_update: "The disturbance now looks tied to an argument over missing property, not just loud music. Every interview makes the apartment feel more like a handoff gone bad than a party.",
    resolution: "You settle the block before rumor hardens into retaliation, and the report captures the conflict underneath the noise instead of writing it off as another messy night.",
    witness_interviews: PATROL_003_WITNESSES,
};
const PATROL_004: CaseFlavorText = CaseFlavorText {
    opening: "A missing pet case lands on your desk because nobody else thinks it matters. The owner insists the animal never bolts without a reason, which turns a soft assignment into a question about what lured it away.",
    mid_case_update: "Tracks and personal items show the pet followed a familiar trail instead of wandering blind. The search is narrowing toward whoever or whatever felt safe enough to chase.",
    resolution: "You bring the runaway search to a humane close and prove that small cases still reveal how well a neighborhood trusts its police.",
    witness_interviews: PATROL_004_WITNESSES,
};
const PATROL_005: CaseFlavorText = CaseFlavorText {
    opening: "A live shoplifting call catches the district at lunchtime, when every exit is crowded and every eyewitness thinks they saw the whole thing. Catching the truth matters more than catching the first runner.",
    mid_case_update: "The theft now reads like a relay instead of a solo grab. The person inside the store mattered less than the calm hand waiting outside to receive the goods.",
    resolution: "You stop a brazen midday theft from becoming another unsolved shrug on downtown's books and show the merchants somebody still pays attention to the details.",
    witness_interviews: PATROL_005_WITNESSES,
};
const PATROL_006: CaseFlavorText = CaseFlavorText {
    opening: "A break-in in the precinct lot gets personal fast because everyone parked there assumes police ground should feel safer than the street. That means the answer has to be better than 'wrong place, wrong time.'",
    mid_case_update: "The damage pattern suggests the car was targeted for something inside, not for the car itself. Whoever hit it knew where surveillance was weakest and expected a quick grab.",
    resolution: "Solving the lot break-in restores a little faith inside the department and reminds the squad that sloppy security embarrasses everyone equally.",
    witness_interviews: PATROL_006_WITNESSES,
};
const PATROL_007: CaseFlavorText = CaseFlavorText {
    opening: "Industrial walls are filling with matching tags, and the city wants them labeled random before anyone asks what they mean. The markings are too disciplined to dismiss that easily.",
    mid_case_update: "The graffiti now looks like territory management dressed up as vandal art. Movement around the drops suggests somebody is using paint to announce logistics.",
    resolution: "You turn a wall-by-wall nuisance into a documented pattern and cut off the message before the district starts believing it belongs to someone else.",
    witness_interviews: PATROL_007_WITNESSES,
};
const PATROL_008: CaseFlavorText = CaseFlavorText {
    opening: "Rail yard management wants a trespasser identified before insurance starts asking why a secured site feels porous. The scene suggests whoever crossed the fence knew exactly when the yard would be half-blind.",
    mid_case_update: "The tracks indicate the intruder moved with purpose rather than curiosity. This was less scavenging and more a timed meeting inside industrial cover.",
    resolution: "You keep the yard incident from being filed as another faceless fence-jump and pin the route down before a riskier return costs someone a life.",
    witness_interviews: PATROL_008_WITNESSES,
};
const DETECTIVE_001: CaseFlavorText = CaseFlavorText {
    opening: "A residential burglary pulls you into the quieter side of fear: a family whose house still looks normal until you notice what privacy now means to them. The offender knew enough routine to arrive between comfort and habit.",
    mid_case_update: "The burglary is tightening around pre-incident surveillance. Someone studied the household first, then counted on embarrassment to keep neighbors vague.",
    resolution: "You clear the burglary with enough specificity that the family gets certainty back instead of a stack of well-meant precautions.",
    witness_interviews: DETECTIVE_001_WITNESSES,
};
const DETECTIVE_002: CaseFlavorText = CaseFlavorText {
    opening: "The downtown assault arrives wrapped in conflicting stories before the bruises have fully risen. Medical evidence will tell a straighter story than frightened bystanders unless you let noise outrun science.",
    mid_case_update: "Forensics are overtaking the crowd narrative. The injury sequence and the camera gaps now suggest a single aggressor who controlled the tempo from the first hit.",
    resolution: "You close the assault by anchoring the case to physical truth, not barstool revisions, and the victim gets more than public sympathy out of the process.",
    witness_interviews: DETECTIVE_002_WITNESSES,
};
const DETECTIVE_003: CaseFlavorText = CaseFlavorText {
    opening: "At first glance the fraud complaint looks like respectable paperwork gone crooked. Then the routed calls, duplicate receipts, and polite silence around the victim make it clear someone built this scheme to survive daylight.",
    mid_case_update: "The paper trail is widening upward. What began as one victim's embarrassment now threatens institutions that trusted the fraudster's discretion more than their own controls.",
    resolution: "You bring the scheme down with records strong enough to survive the city's urge to call it an accounting problem instead of a crime.",
    witness_interviews: DETECTIVE_003_WITNESSES,
};
const DETECTIVE_004: CaseFlavorText = CaseFlavorText {
    opening: "The missing person file has the usual dead air around it, but the timeline feels staged. Every ordinary movement in the days before the disappearance now looks like either caution or pressure.",
    mid_case_update: "Interviews and traces are sketching a controlled exit rather than a clean voluntary one. Someone kept the victim moving just enough to muddy intent.",
    resolution: "You turn a fading absence back into a solved narrative and spare the family the long cruelty of never knowing whether to grieve or wait.",
    witness_interviews: DETECTIVE_004_WITNESSES,
};
const DETECTIVE_005: CaseFlavorText = CaseFlavorText {
    opening: "The warehouse fire is being sold as bad luck, bad wiring, and bad timing. The scene disagrees; it smells like a planned burn meant to outrun questions about what was inside before the smoke.",
    mid_case_update: "Records and forensics now align around preparation. Inventory shifted, wind was exploited, and the blaze begins to look like a business decision disguised as misfortune.",
    resolution: "You resolve the arson with enough supporting detail that nobody gets to hide profit behind ash and condolence language.",
    witness_interviews: DETECTIVE_005_WITNESSES,
};
const DETECTIVE_006: CaseFlavorText = CaseFlavorText {
    opening: "A possession ring case tempts the precinct into counting arrests instead of structure. The smarter play is to ignore the easy collars and read who keeps cycling through them untouched.",
    mid_case_update: "The ring is revealing its shape through repetition. Small-time possession reports now look like camouflage for a cleaner, more disciplined distribution network.",
    resolution: "You close on the ring's connective tissue instead of its disposable edges, which means the bust actually changes the street for a while.",
    witness_interviews: DETECTIVE_006_WITNESSES,
};
const DETECTIVE_007: CaseFlavorText = CaseFlavorText {
    opening: "The hit and run scene is brutal in the way fast roads often are: one instant of noise, then a trail that starts disappearing with every passing truck. You have just enough trace evidence to keep the driver from rewriting history in a body shop.",
    mid_case_update: "Vehicle class, damage pattern, and hesitation marks all suggest the driver knew exactly what happened and chose flight over aid after a measurable pause.",
    resolution: "You tie the crash to a driver instead of a mythic 'unknown vehicle' and give the victim's file a human answer instead of highway statistics.",
    witness_interviews: DETECTIVE_007_WITNESSES,
};
const DETECTIVE_008: CaseFlavorText = CaseFlavorText {
    opening: "The blackmail case arrives through people who are more offended than frightened, which usually means the leverage is real. Somebody knows which secret matters and how far institutional pride will go to bury it.",
    mid_case_update: "Financial and communication records point past a lone extortionist toward someone fluent in reputational pressure. The blackmail is targeted, timed, and legally literate.",
    resolution: "You end the squeeze with evidence sturdy enough that power can't redefine victimhood once the spotlight hits.",
    witness_interviews: DETECTIVE_008_WITNESSES,
};
const SERGEANT_001: CaseFlavorText = CaseFlavorText {
    opening: "A homicide downtown turns the precinct from reactive to scrutinized in a single shift. Every choice now matters twice: once for the victim, and once for a city already deciding whether you can carry the case.",
    mid_case_update: "The scene is tightening around a planned confrontation with managed witness visibility. Somebody shaped the streets before the killing, not just the escape after it.",
    resolution: "You close the homicide with a case sturdy enough to survive grief, politics, and defense strategy, which is the closest thing this town gets to justice sticking.",
    witness_interviews: SERGEANT_001_WITNESSES,
};
const SERGEANT_002: CaseFlavorText = CaseFlavorText {
    opening: "The kidnapping file gives you almost no time to be wrong. Witness memory, vehicle movement, and family panic are all racing the same clock, and only one of them leaves evidence.",
    mid_case_update: "The abduction now appears rehearsed rather than impulsive. The vehicle, route interference, and school-adjacent surveillance suggest an offender who counted on ordinary routines to do half the work.",
    resolution: "You bring the child home by turning fragmented panic into a usable timeline, and the city sees a rare case where urgency did not devour accuracy.",
    witness_interviews: SERGEANT_002_WITNESSES,
};
const SERGEANT_003: CaseFlavorText = CaseFlavorText {
    opening: "What looked like unrelated thefts are starting to resolve into an actual ring with scheduling, storage, and fallback routes. That means every small report you ignored earlier just got heavier.",
    mid_case_update: "Thefts across districts now read like coordinated logistics. Merchandise moves through predictable cover, and lower-level operators appear terrified of whoever handles the books.",
    resolution: "You collapse the ring as a network instead of collecting random thieves, which keeps the win from dissolving into replacements by next week.",
    witness_interviews: SERGEANT_003_WITNESSES,
};
const SERGEANT_004: CaseFlavorText = CaseFlavorText {
    opening: "Corruption cases are ugly because every good lead arrives wrapped in institutional shame. The evidence points inward, and every correct step risks teaching the wrong cops how to hide better next time.",
    mid_case_update: "Financial records, coached reports, and legal whispers are converging on a system rather than a bad apple. Someone has been laundering confidence through process.",
    resolution: "You land the corruption case cleanly enough that the department has to face what happened instead of blaming the investigation for noticing it.",
    witness_interviews: SERGEANT_004_WITNESSES,
};
const SERGEANT_005: CaseFlavorText = CaseFlavorText {
    opening: "A cold case reopens because modern evidence refuses to respect old certainty. The file is dusty, the witnesses are tired, and every original mistake is now part of the crime scene.",
    mid_case_update: "Fresh forensics are exposing whether the old investigation failed from age, incompetence, or design. The reopened timeline is turning yesterday's confidence into today's liability.",
    resolution: "You give an old wound a current answer and prove the precinct can revisit its own history without flinching away from what it finds.",
    witness_interviews: SERGEANT_005_WITNESSES,
};
const SERGEANT_006: CaseFlavorText = CaseFlavorText {
    opening: "The serial vandal case matters because the city wants it not to. Pattern crimes erode confidence slowly, and someone has been counting on residents to keep calling the damage random.",
    mid_case_update: "Scene selection now reveals motive through vulnerability. The offender is choosing places that cannot afford attention and counting on that silence to continue.",
    resolution: "You stop the vandal pattern before it matures into accepted background fear, which is often the real victory in this kind of case.",
    witness_interviews: SERGEANT_006_WITNESSES,
};
const LIEUTENANT_001: CaseFlavorText = CaseFlavorText {
    opening: "Multiple murders have become one investigation whether city hall is ready to say so or not. The pattern is there, and the only question left is whether you can get ahead of someone who learns from every body.",
    mid_case_update: "The killer's behavioral pattern now intersects with geography, cooling-off periods, and scene confidence. This is no longer a loose theory; it is a narrowing corridor.",
    resolution: "You close the serial investigation by proving pattern with discipline, denying the killer the myth they were building out of repetition and public dread.",
    witness_interviews: LIEUTENANT_001_WITNESSES,
};
const LIEUTENANT_002: CaseFlavorText = CaseFlavorText {
    opening: "The conspiracy case starts as whispers between unrelated offices and ends with the possibility that they were never unrelated at all. Every ledger and letter now asks how long the city has been coordinating its own decay.",
    mid_case_update: "The network is surfacing through administrative seams: budgets, schedules, donor contact, and selective enforcement all pointing to shared management rather than coincidence.",
    resolution: "You expose the conspiracy as a system with names, records, and connective tissue, which keeps the truth from being diluted into a few convenient resignations.",
    witness_interviews: LIEUTENANT_002_WITNESSES,
};
const LIEUTENANT_003: CaseFlavorText = CaseFlavorText {
    opening: "The final case gathers every favor, betrayal, and half-seen pattern of your career into one live file. If you rush it, the city will survive by making you the problem instead of the proof.",
    mid_case_update: "The evidence now links institutions that spent the whole game pretending their failures were isolated. Every interview is confirming that this ending was built long before you saw the outline.",
    resolution: "You close the final case by holding every thread together long enough for the truth to become unavoidable, and the rookie's story ends as a hard-earned piece of institutional memory.",
    witness_interviews: LIEUTENANT_003_WITNESSES,
};

pub(crate) fn case_flavor(case_id: &str) -> Option<&'static CaseFlavorText> {
    match case_id {
        "patrol_001_petty_theft" => Some(&PATROL_001),
        "patrol_002_vandalism" => Some(&PATROL_002),
        "patrol_003_noise" => Some(&PATROL_003),
        "patrol_004_lost_pet" => Some(&PATROL_004),
        "patrol_005_shoplifting" => Some(&PATROL_005),
        "patrol_006_car_breakin" => Some(&PATROL_006),
        "patrol_007_graffiti" => Some(&PATROL_007),
        "patrol_008_trespassing" => Some(&PATROL_008),
        "detective_001_burglary" => Some(&DETECTIVE_001),
        "detective_002_assault" => Some(&DETECTIVE_002),
        "detective_003_fraud" => Some(&DETECTIVE_003),
        "detective_004_missing" => Some(&DETECTIVE_004),
        "detective_005_arson" => Some(&DETECTIVE_005),
        "detective_006_drugs" => Some(&DETECTIVE_006),
        "detective_007_hitrun" => Some(&DETECTIVE_007),
        "detective_008_blackmail" => Some(&DETECTIVE_008),
        "sergeant_001_homicide" => Some(&SERGEANT_001),
        "sergeant_002_kidnapping" => Some(&SERGEANT_002),
        "sergeant_003_theft_ring" => Some(&SERGEANT_003),
        "sergeant_004_corruption" => Some(&SERGEANT_004),
        "sergeant_005_cold_case" => Some(&SERGEANT_005),
        "sergeant_006_serial_vandal" => Some(&SERGEANT_006),
        "lieutenant_001_serial" => Some(&LIEUTENANT_001),
        "lieutenant_002_conspiracy" => Some(&LIEUTENANT_002),
        "lieutenant_003_final" => Some(&LIEUTENANT_003),
        _ => None,
    }
}

pub(crate) fn witness_lines(case_id: &str, witness_id: &str) -> Option<&'static [&'static str]> {
    case_flavor(case_id)?
        .witness_interviews
        .iter()
        .find(|interview| interview.witness_id == witness_id)
        .map(|interview| interview.lines)
}

pub(crate) fn witness_ids(case_id: &str) -> Vec<NpcId> {
    case_flavor(case_id)
        .map(|flavor| {
            flavor
                .witness_interviews
                .iter()
                .map(|interview| interview.witness_id.to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_precinct_case_has_full_flavor_text() {
        for case_id in [
            "patrol_001_petty_theft",
            "patrol_002_vandalism",
            "patrol_003_noise",
            "patrol_004_lost_pet",
            "patrol_005_shoplifting",
            "patrol_006_car_breakin",
            "patrol_007_graffiti",
            "patrol_008_trespassing",
            "detective_001_burglary",
            "detective_002_assault",
            "detective_003_fraud",
            "detective_004_missing",
            "detective_005_arson",
            "detective_006_drugs",
            "detective_007_hitrun",
            "detective_008_blackmail",
            "sergeant_001_homicide",
            "sergeant_002_kidnapping",
            "sergeant_003_theft_ring",
            "sergeant_004_corruption",
            "sergeant_005_cold_case",
            "sergeant_006_serial_vandal",
            "lieutenant_001_serial",
            "lieutenant_002_conspiracy",
            "lieutenant_003_final",
        ] {
            let flavor = case_flavor(case_id).unwrap();
            assert!(!flavor.opening.is_empty());
            assert!(!flavor.mid_case_update.is_empty());
            assert!(!flavor.resolution.is_empty());
            assert!(!flavor.witness_interviews.is_empty());
            assert!(flavor
                .witness_interviews
                .iter()
                .all(|interview| interview.lines.len() >= 2));
        }
    }
}
