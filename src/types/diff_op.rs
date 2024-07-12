pub enum DiffOp {
    CreateSfn,
    CreateSchedule,
    UpdateSfn,
    UpdateSchedule,
    AddSfnTag,
    AddScheduleTag,
    RemoveSfnTag,
    RemoveScheduleTag,
    DeleteSfn,
    DeleteSchedule,
    NoChangeSfn,
    NoChangeSchedule,
}
