/// Track the execution of a given Flow.
#[derive(Deserialize, Serialize, Debug)]
pub struct FlowRun {
    name: String,
    start_time: String,
    end_time: String,
}

/// High level Milestone being worked on.
#[derive(Deserialize, Serialize, Debug)]
pub struct Epic {
    name: String,
    start_time: String,
    end_time: String,
    flows: Vec<FlowRun>
}