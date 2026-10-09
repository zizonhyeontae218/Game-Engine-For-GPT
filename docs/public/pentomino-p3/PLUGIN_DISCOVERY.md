# Structured discovery and installation

Call ViewHost::describe() on a new host: available_formats advertises Classic2D,
TopDown, Side and Vertical; projections and behaviors advertise native camera
capabilities. Required semantic fields live in each FormatDescriptor;
RecordBinding maps them to the consumer's owner/schema/field names.

Install one linked FormatPlugin with a ViewConfig and actual Core-selected
ReadFrame. initialize is part of install. Add any bounded number of distinct
CameraConfig values referencing that View, then activate an explicit set.
No singleton camera exists. describe() reports installed configs, camera IDs,
active flags and current transition status. Missing resources are errors.

CameraConfig::new defaults to Smooth12ticks; CameraChange with Instant opts out.
All camera/viewing-mode changes are camera-local. Changes target the next
presentation tick; presentation ticks need not advance authoritative Core.
Remove/replace does not mutate the supplied ReadFrame or Core. A legacy source
importer is a separate authority that initializes ordinary Core records once;
it is not a continuously delegated gameplay runtime.
