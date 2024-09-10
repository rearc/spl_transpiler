mod regex_utils;
#[allow(unused_imports)]
/* Full command list: https://docs.splunk.com/Documentation/SplunkCloud/latest/SearchReference/ListOfSearchCommands
Command          	 Description
abstract         	 Produces a summary of each search result.
accum            	 Keeps a running total of the specified numeric field.
addcoltotals     	 Computes an event that contains sum of all numeric fields for previous events.
addinfo          	 Add fields that contain common information about the current search.
addtotals        	 Computes the sum of all numeric fields for each result.
analyzefields    	 Analyze numerical fields for their ability to predict another discrete field.
anomalies        	 Computes an "unexpectedness" score for an event.
anomalousvalue   	 Finds and summarizes irregular, or uncommon, search results.
anomalydetection 	 Identifies anomalous events by computing a probability for each event and then detecting unusually small probabilities.
append           	 Appends subsearch results to current results.
appendcols       	 Appends the fields of the subsearch results to current results, first results to first result, second to second, etc.
appendpipe       	 Appends the result of the subpipeline applied to the current result set to results.
arules           	 Finds association rules between field values.
associate        	 Identifies correlations between fields.
autoregress      	 Sets up data for calculating the moving average.
bin (bucket)     	 Puts continuous numerical values into discrete sets.
bucketdir        	 Replaces a field value with higher-level grouping, such as replacing filenames with directories.
chart            	 Returns results in a tabular output for charting. See also, Statistical and charting functions.
cluster          	 Clusters similar events together.
cofilter         	 Finds how many times field1 and field2 values occurred together.
collect          	 Puts search results into a summary index.
concurrency      	 Uses a duration field to find the number of "concurrent" events for each event.
contingency      	 Builds a contingency table for two fields.
convert          	 Converts field values into numerical values.
correlate        	 Calculates the correlation between different fields.
datamodel        	 Examine data model or data model dataset and search a data model dataset.
dbinspect        	 Returns information about the specified index.
dedup            	 Removes subsequent results that match a specified criteria.
delete           	 Delete specific events or search results.
delta            	 Computes the difference in field value between nearby results.
diff             	 Returns the difference between two search results.
erex             	 Allows you to specify example or counter example values to automatically extract fields that have similar values.
eval             	 Calculates an expression and puts the value into a field. See also, Evaluation functions.
eventcount       	 Returns the number of events in an index.
eventstats       	 Adds summary statistics to all search results.
extract (kv)     	 Extracts field-value pairs from search results.
fieldformat      	 Expresses how to render a field at output time without changing the underlying value.
fields           	 Keeps or removes fields from search results based on the field list criteria.
fieldsummary     	 Generates summary information for all or a subset of the fields.
filldown         	 Replaces NULL values with the last non-NULL value.
fillnull         	 Replaces null values with a specified value.
findtypes        	 Generates a list of suggested event types.
folderize        	 Creates a higher-level grouping, such as replacing filenames with directories.
foreach          	 Run a templatized streaming subsearch for each field in a wildcarded field list.
format           	 Takes the results of a subsearch and formats them into a single result.
from             	 Retrieves data from a dataset, such as a data model dataset, a CSV lookup, a KV Store lookup, a saved search, or a table dataset.
gauge            	 Transforms results into a format suitable for display by the Gauge chart types.
gentimes         	 Generates time-range results.
geom             	 Adds a field, named geom, to each event. This field contains geographic data structures for polygon geometry in JSON and is used for the choropleth map visualization.
geomfilter       	 Accepts two points that specify a bounding box for clipping a choropleth map. Points that fall outside of the bounding box are filtered out.
geostats         	 Generate statistics which are clustered into geographical bins to be rendered on a world map.
head             	 Returns the first number n of specified results.
highlight        	 Highlights the specified terms.
history          	 Returns a history of searches formatted as an events list or as a table.
iconify          	 Displays a unique icon for each different value in the list of fields that you specify.
inputcsv         	 Loads search results from the specified CSV file.
inputlookup      	 Loads search results from a specified static lookup table.
iplocation       	 Extracts location information from IP addresses.
join             	 Combine the results of a subsearch with the results of a main search.
kmeans           	 Performs k-means clustering on selected fields.
kvform           	 Extracts values from search results, using a form template.
loadjob          	 Loads events or results of a previously completed search job.
localize         	 Returns a list of the time ranges in which the search results were found.
localop          	 Run subsequent commands, that is all commands following this, locally and not on remote peers.
lookup           	 Explicitly invokes field value lookups.
makecontinuous   	 Makes a field that is supposed to be the x-axis continuous (invoked by chart/timechart)
makemv           	 Change a specified field into a multivalued field during a search.
makeresults      	 Creates a specified number of empty search results.
map              	 A looping operator, performs a search over each search result.
mcollect         	 Converts search results into metric data and inserts the data into a metric index on the search head.
metadata         	 Returns a list of source, sourcetypes, or hosts from a specified index or distributed search peer.
metasearch       	 Retrieves event metadata from indexes based on terms in the logical expression.
meventcollect    	 Converts search results into metric data and inserts the data into a metric index on the indexers.
mpreview         	 Returns a preview of the raw metric data points in a specified metric index that match a provided filter.
msearch          	 Alias for the mpreview command.
mstats           	 Calculates statistics for the measurement, metric_name, and dimension fields in metric indexes.
multikv          	 Extracts field-values from table-formatted events.
multisearch      	 Run multiple streaming searches at the same time.
mvcombine        	 Combines events in search results that have a single differing field value into one result with a multivalue field of the differing field.
mvexpand         	 Expands the values of a multivalue field into separate events for each value of the multivalue field.
nomv             	 Changes a specified multivalued field into a single-value field at search time.
outlier          	 Removes outlying numerical values.
outputcsv        	 Outputs search results to a specified CSV file.
outputlookup     	 Writes search results to the specified static lookup table.
outputtext       	 Outputs the raw text field (_raw) of results into the _xml field.
overlap          	 Finds events in a summary index that overlap in time or have missed events.
pivot            	 Run pivot searches against a particular data model dataset.
predict          	 Enables you to use time series algorithms to predict future values of fields.
rangemap         	 Sets RANGE field to the name of the ranges that match.
rare             	 Displays the least common values of a field.
redistribute     	 Implements parallel reduce search processing to shorten the search runtime of high-cardinality dataset searches.
regex            	 Removes results that do not match the specified regular expression.
reltime          	 Converts the difference between 'now' and '_time' to a human-readable value and adds adds this value to the field, 'reltime', in your search results.
rename           	 Renames a specified field; wildcards can be used to specify multiple fields.
replace          	 Replaces values of specified fields with a specified new value.
require          	 Causes a search to fail if the queries and commands that precede it in the search string return zero events or results.
rest             	 Access a REST endpoint and display the returned entities as search results.
return           	 Specify the values to return from a subsearch.
reverse          	 Reverses the order of the results.
rex              	 Specify a Perl regular expression named groups to extract fields while you search.
rtorder          	 Buffers events from real-time search to emit them in ascending time order when possible.
savedsearch      	 Returns the search results of a saved search.
script (run)     	 Runs an external Perl or Python script as part of your search.
scrub            	 Anonymizes the search results.
search           	 Searches indexes for matching events.
searchtxn        	 Finds transaction events within specified search constraints.
selfjoin         	 Joins results with itself.
sendalert        	 invokes a custom alert action.
sendemail        	 Emails search results to a specified email address.
set              	 Performs set operations (union, diff, intersect) on subsearches.
setfields        	 Sets the field values for all results to a common value.
sichart          	 Summary indexing version of the chart command.
sirare           	 Summary indexing version of the rare command.
sistats          	 Summary indexing version of the stats command.
sitimechart      	 Summary indexing version of the timechart command.
sitop            	 Summary indexing version of the top command.
sort             	 Sorts search results by the specified fields.
spath            	 Provides a straightforward means for extracting fields from structured data formats, XML and JSON.
stats            	 Provides statistics, grouped optionally by fields. See also, Statistical and charting functions.
strcat           	 Concatenates string values.
streamstats      	 Adds summary statistics to all search results in a streaming manner.
table            	 Creates a table using the specified fields.
tags             	 Annotates specified fields in your search results with tags.
tail             	 Returns the last number n of specified results.
timechart        	 Create a time series chart and corresponding table of statistics. See also, Statistical and charting functions.
timewrap         	 Displays, or wraps, the output of the timechart command so that every timewrap-span range of time is a different series.
tojson           	 Converts events into JSON objects.
top              	 Displays the most common values of a field.
transaction      	 Groups search results into transactions.
transpose        	 Reformats rows of search results as columns.
trendline        	 Computes moving averages of fields.
tscollect        	 Writes results into tsidx file(s) for later use by the tstats command.
tstats           	 Calculates statistics over tsidx files created with the tscollect command.
typeahead        	 Returns typeahead information on a specified prefix.
typelearner      	 Deprecated. Use findtypes instead. Generates suggested eventtypes.
typer            	 Calculates the eventtypes for the search results.
union            	 Merges the results from two or more datasets into one dataset.
uniq             	 Removes any search that is an exact duplicate with a previous result.
untable          	 Converts results from a tabular format to a format similar to stats output. Inverse of xyseries and maketable.
walklex          	 Generates a list of terms or indexed fields from each bucket of event indexes.
where            	 Performs arbitrary filtering on your data. See also, Evaluations functions.
x11              	 Enables you to determine the trend in your data by removing the seasonal pattern.
xmlkv            	 Extracts XML key-value pairs.
xmlunescape      	 Unescapes XML.
xpath            	 Redefines the XML path.
xyseries         	 Converts results into a format suitable for graphing.
 */
pub mod spl;

use const_str::replace;
use paste::paste;
use pyo3::prelude::*;

pub trait CommandBase {
    const NAME: &'static str;
    const ALIAS: Option<&'static str>;
}

macro_rules! make_command {
    ($name:ident, $attrs:tt) => {
        paste!{
            make_command!(
                _,
                [<cmd_ $name>],
                [<$name:camel Parser>],
                [<$name:camel CommandRoot>],
                replace!(stringify!($name), "_", ""),
                None,
                $attrs
            );
        }
    };
    ($name:ident ( $alias:ident ) , $attrs:tt) => {
        paste!{
            make_command!(
                _,
                [<cmd_ $name>],
                [<$name:camel Parser>],
                [<$name:camel CommandRoot>],
                replace!(stringify!($name), "_", ""),
                Some(replace!(stringify!($alias), "_", "")),
                $attrs
            );
        }
    };
    (_, $module_name:ident, $parser_name:ident, $command_name:ident, $name:expr, $alias:expr, { $($attr_name:ident : $attr_type:ty),* }) => {
        pub mod $module_name;

        #[derive(Debug, Clone, PartialEq, Hash)]
        #[pyclass(frozen, eq, hash)]
        pub struct $command_name {
            $(pub $attr_name: $attr_type),*
        }

        impl CommandBase for $command_name {
            const NAME: &'static str = $name;
            const ALIAS: Option<&'static str> = $alias;
        }
    };
}

/*
pub enum Command {
    Abstract(AbstractCommand),
    Accum(AccumCommand),
    AddColTotals(AddColTotalsCommand),
    AddInfo(AddInfoCommand),
    AddTotals(AddTotalsCommand),
    AnalyzeFields(AnalyzeFieldsCommand),
    Anomalies(AnomaliesCommand),
    AnomalousValue(AnomalousValueCommand),
    AnomalyDetection(AnomalyDetectionCommand),
    Append(AppendCommand),
    AppendCols(AppendColsCommand),
    AppendPipe(AppendPipeCommand),
    Arules(ArulesCommand),
    Associate(AssociateCommand),
    AutoRegress(AutoRegressCommand),
    Bin(BinCommand),
    BucketDir(BucketDirCommand),
    Chart(ChartCommand),
    Cluster(ClusterCommand),
    CoFilter(CoFilterCommand),
    Collect(CollectCommand),
    Concurrency(ConcurrencyCommand),
    Contingency(ContingencyCommand),
    Convert(ConvertCommand),
    Correlate(CorrelateCommand),
    DataModel(DataModelCommand),
    DbInspect(DbInspectCommand),
    Dedup(DedupCommand),
    Delete(DeleteCommand),
    Delta(DeltaCommand),
    Diff(DiffCommand),
    Erex(ErexCommand),
    Eval(EvalCommand),
    EventCount(EventCountCommand),
    EventStats(EventStatsCommand),
    Extract(ExtractCommand),
    FieldFormat(FieldFormatCommand),
    Fields(FieldsCommand),
    FieldSummary(FieldSummaryCommand),
    FillDown(FillDownCommand),
    FillNull(FillNullCommand),
    FindTypes(FindTypesCommand),
    Folderize(FolderizeCommand),
    ForEach(ForEachCommand),
    Format(FormatCommand),
    From(FromCommand),
    Gauge(GaugeCommand),
    GenTimes(GenTimesCommand),
    Geom(GeomCommand),
    GeomFilter(GeomFilterCommand),
    GeoStats(GeoStatsCommand),
    Head(HeadCommand),
    Highlight(HighlightCommand),
    History(HistoryCommand),
    Iconify(IconifyCommand),
    InputCsv(InputCsvCommand),
    InputLookup(InputLookupCommand),
    IpLocation(IpLocationCommand),
    Join(JoinCommand),
    KMeans(KMeansCommand),
    KvForm(KvFormCommand),
    LoadJob(LoadJobCommand),
    Localize(LocalizeCommand),
    LocalOp(LocalOpCommand),
    Lookup(LookupCommand),
    MakeContinuous(MakeContinuousCommand),
    MakeMv(MakeMvCommand),
    MakeResults(MakeResultsCommand),
    Map(MapCommand),
    MCollect(MCollectCommand),
    Metadata(MetadataCommand),
    MetaSearch(MetaSearchCommand),
    MEventCollect(MEventCollectCommand),
    MPreview(MPreviewCommand),
    MSearch(MSearchCommand),
    MStats(MStatsCommand),
    MultiKv(MultiKvCommand),
    MultiSearch(MultiSearchCommand),
    MvCombine(MvCombineCommand),
    MvExpand(MvExpandCommand),
    NoMv(NoMvCommand),
    Outlier(OutlierCommand),
    OutputCsv(OutputCsvCommand),
    OutputLookup(OutputLookupCommand),
    OutputText(OutputTextCommand),
    Overlap(OverlapCommand),
    Pivot(PivotCommand),
    Predict(PredictCommand),
    RangeMap(RangeMapCommand),
    Rare(RareCommand),
    Redistribute(RedistributeCommand),
    Regex(RegexCommand),
    RelTime(RelTimeCommand),
    Rename(RenameCommand),
    Replace(ReplaceCommand),
    Require(RequireCommand),
    Rest(RestCommand),
    Return(ReturnCommand),
    Reverse(ReverseCommand),
    Rex(RexCommand),
    RtOrder(RtOrderCommand),
    SavedSearch(SavedSearchCommand),
    Script(ScriptCommand),
    Scrub(ScrubCommand),
    Search(SearchCommand),
    SearchTxn(SearchTxnCommand),
    SelfJoin(SelfJoinCommand),
    SendAlert(SendAlertCommand),
    SendEmail(SendEmailCommand),
    Set(SetCommand),
    SetFields(SetFieldsCommand),
    SiChart(SiChartCommand),
    SiRare(SiRareCommand),
    SiStats(SiStatsCommand),
    SiTimeChart(SiTimeChartCommand),
    SiTop(SiTopCommand),
    Sort(SortCommand),
    SPath(SPathCommand),
    Stats(StatsCommand),
    StrCat(StrCatCommand),
    StreamStats(StreamStatsCommand),
    Table(TableCommand),
    Tags(TagsCommand),
    Tail(TailCommand),
    TimeChart(TimeChartCommand),
    TimeWrap(TimeWrapCommand),
    ToJson(ToJsonCommand),
    Top(TopCommand),
    Transaction(TransactionCommand),
    Transpose(TransposeCommand),
    TrendLine(TrendLineCommand),
    TsCollect(TsCollectCommand),
    TStats(TStatsCommand),
    TypeAhead(TypeAheadCommand),
    TypeLearner(TypeLearnerCommand),
    Typer(TyperCommand),
    Union(UnionCommand),
    Uniq(UniqCommand),
    UnTable(UnTableCommand),
    WalkLex(WalkLexCommand),
    Where(WhereCommand),
    X11(X11Command),
    XmlKv(XmlKvCommand),
    XmlUnescape(XmlUnescapeCommand),
    Xpath(XpathCommand),
    XySeries(XySeriesCommand),
}
 */

// make_command!(abstract, {});
// make_command!(accum, {});
// make_command!(add_col_totals, {});
// make_command!(add_info, {});
make_command!(add_totals, {});
// make_command!(analyze_fields, {});
// make_command!(anomalies, {});
// make_command!(anomalous_value, {});
// make_command!(anomaly_detection, {});
// make_command!(append, {});
// make_command!(append_cols, {});
// make_command!(append_pipe, {});
// make_command!(arules, {});
// make_command!(associate, {});
// make_command!(auto_regress, {});
make_command!(bin(bucket), {});
// make_command!(bucket_dir, {});
// make_command!(chart, {});
// make_command!(cluster, {});
// make_command!(co_filter, {});
make_command!(collect, {});
// make_command!(concurrency, {});
// make_command!(contingency, {});
make_command!(convert, {});
// make_command!(correlate, {});
// make_command!(data_model, {});
// make_command!(db_inspect, {});
make_command!(dedup, {});
// make_command!(delete, {});
// make_command!(delta, {});
// make_command!(diff, {});
// make_command!(erex, {});
make_command!(eval, {});
// make_command!(event_count, {});
make_command!(event_stats, {});
// make_command!(extract (kv), {});
// make_command!(field_format, {});
make_command!(fields, {});
// make_command!(field_summary, {});
// make_command!(fill_down, {});
make_command!(fill_null, {});
// make_command!(find_types, {});
// make_command!(folderize, {});
// make_command!(for_each, {});
make_command!(format, {});
// make_command!(from, {});
// make_command!(gauge, {});
// make_command!(gen_times, {});
// make_command!(geom, {});
// make_command!(geom_filter, {});
// make_command!(geo_stats, {});
make_command!(head, {});
// make_command!(highlight, {});
// make_command!(history, {});
// make_command!(iconify, {});
// make_command!(input_csv, {});
make_command!(input_lookup, {});
// make_command!(ip_location, {});
make_command!(join, {});
// make_command!(k_means, {});
// make_command!(kv_form, {});
// make_command!(load_job, {});
// make_command!(localize, {});
// make_command!(local_op, {});
make_command!(lookup, {});
// make_command!(make_continuous, {});
// make_command!(make_mv, {});
make_command!(make_results, {});
make_command!(map, {});
// make_command!(m_collect, {});
// make_command!(metadata, {});
// make_command!(meta_search, {});
// make_command!(m_event_collect, {});
// make_command!(m_preview, {});
// make_command!(m_search, {});
// make_command!(m_stats, {});
// make_command!(multi_kv, {});
make_command!(multi_search, {});
make_command!(mv_combine, {});
make_command!(mv_expand, {});
// make_command!(no_mv, {});
// make_command!(outlier, {});
// make_command!(output_csv, {});
// make_command!(output_lookup, {});
// make_command!(output_text, {});
// make_command!(overlap, {});
// make_command!(pivot, {});
// make_command!(predict, {});
// make_command!(range_map, {});
// make_command!(rare, {});
// make_command!(redistribute, {});
make_command!(regex, {});
// make_command!(rel_time, {});
make_command!(rename, {});
// make_command!(replace, {});
// make_command!(require, {});
// make_command!(rest, {});
make_command!(return, {});
// make_command!(reverse, {});
make_command!(rex, {});
// make_command!(rt_order, {});
// make_command!(saved_search, {});
// make_command!(script (run), {});
// make_command!(scrub, {});
make_command!(search, {});
// make_command!(search_txn, {});
// make_command!(self_join, {});
// make_command!(send_alert, {});
// make_command!(send_email, {});
// make_command!(set, {});
// make_command!(set_fields, {});
// make_command!(si_chart, {});
// make_command!(si_rare, {});
// make_command!(si_stats, {});
// make_command!(si_time_chart, {});
// make_command!(si_top, {});
make_command!(sort, {});
// make_command!(s_path, {});
make_command!(stats, {});
// make_command!(str_cat, {});
make_command!(stream_stats, {});
make_command!(table, {});
// make_command!(tags, {});
// make_command!(tail, {});
// make_command!(time_chart, {});
// make_command!(time_wrap, {});
// make_command!(to_json, {});
make_command!(top, {});
// make_command!(transaction, {});
// make_command!(transpose, {});
// make_command!(trend_line, {});
// make_command!(ts_collect, {});
// make_command!(t_stats, {});
// make_command!(type_ahead, {});
// make_command!(type_learner, {});
// make_command!(typer, {});
// make_command!(union, {});
// make_command!(uniq, {});
// make_command!(un_table, {});
// make_command!(walk_lex, {});
make_command!(where, {});
// make_command!(x11, {});
// make_command!(xml_kv, {});
// make_command!(xml_unescape, {});
// make_command!(xpath, {});
// make_command!(xy_series, {});
