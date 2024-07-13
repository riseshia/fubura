pub enum DiffOp {
    CreateSfn,
    CreateSchedule,
    UpdateSfn,
    UpdateSchedule,
    AddSfnTag,
    RemoveSfnTag,
    DeleteSfn,
    DeleteSchedule,
    NoChangeSfn,
    NoChangeSfnTags,
    NoChangeSchedule,
}
