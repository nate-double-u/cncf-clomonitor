/**
 * Interface implemented by all organizations
 */
export interface Organization {
  name: string;
  display_name?: string;
  description?: string;
  home_url: string;
  logo_url?: string;
}

/**
 * Interface implemented as project base
 */
export interface BaseProject {
  id: string;
  name: string;
  display_name?: string;
  description?: string;
  home_url: string;
  logo_url?: string;
  devstats_url?: string;
  maturity_id: Maturity;
  category_id: Category;
  score: Score;
  updated_at: number;
}

/**
 * Interface implemented by projects in search list
 */
export interface Project extends BaseProject {
  repositories: BaseRepository[];
  organization: {
    name: string;
  };
}

/**
 * Interface implemented by projects in detail page
 */
export interface ProjectDetail extends BaseProject {
  repositories: Repository[];
}

/**
 * Interface implemented as repository base
 */
export interface BaseRepository {
  name: string;
  url: string;
  kind: RepositoryKind;
}

/**
 * Interface implemented by repository used in {@link ProjectDetail | project detail}
 */
export interface Repository extends BaseRepository {
  digest: string;
  repository_id: string;
  score: Score;
  reports: Report[];
}

/**
 * Interface implemented by all reports
 */
export interface Report {
  data: CoreReport | any;
  linter_id: LinterId;
  report_id: string;
  updated_at: number;
}

/**
 * Interface implemented by core report - first key is {@link ScoreType}
 */
export interface CoreReport {
  // ScoreType
  [key: string]: {
    [key: string]: number | boolean;
  };
}

/**
 * Interface implemented by Score in {@link Repository} and {@link BaseProject}
 */
export type Score = {
  [key in ScoreType]: number;
} & { score_kind: ScoreKind };

/**
 * Interface implemented by all filters sections
 */
export interface FiltersSection {
  name: string;
  title: string;
  filters: Filter[];
}

/**
 * Interface implemented by all filters
 */
export interface Filter {
  name: string | number;
  label: string;
  legend?: string;
  decorator?: JSX.Element;
}

/**
 * Interface impletemed by all issues
 */
export interface Issue {
  level: number;
  description: string;
}

/**
 * Interface implemented by localstorage prefs
 */
export interface Prefs {
  search: { limit: number; sort: { by: SortBy; direction: SortDirection } };
  theme: {
    effective: string;
  };
}

/**
 * Interface implemented by all report checks
 */
export interface ReportOptionData {
  icon: JSX.Element;
  name: string;
  legend: JSX.Element;
  description: JSX.Element;
}

/**
 * Enum implemeted by maturity level
 */
export enum Maturity {
  Graduated = 0,
  Incubating,
  Sandbox,
}

/**
 * Enum implemented by project category
 */
export enum Category {
  'App definition' = 0,
  Observability,
  Orchestration,
  Platform,
  Provisioning,
  Runtime,
  Serverless,
}

/**
 * Enum implemented by quality rating
 */
export enum Rating {
  A = 'a',
  B = 'b',
  C = 'c',
  D = 'd',
}

/**
 * Enum implemented by filter kind
 */
export enum FilterKind {
  Maturity = 'maturity',
  Category = 'category',
  Rating = 'rating',
}

/**
 * Enum implemented by linter kind
 */
export enum LinterId {
  core = 0,
}

/**
 * Enum implemented by checks category
 */
export enum ScoreType {
  BestPractices = 'best_practices',
  Documentation = 'documentation',
  Global = 'global',
  Legal = 'legal',
  License = 'license',
  Security = 'security',
}

/**
 * Enum implemented by sort direction
 */
export enum SortDirection {
  ASC = 'asc',
  DESC = 'desc',
}

/**
 * Enum implemented by sort criteria
 */
export enum SortBy {
  Name = 'name',
  Score = 'score',
}

/**
 * Enum implemented by reporitory kind
 */
export enum RepositoryKind {
  Primary = 'primary',
  Secondary = 'secondary',
}

/**
 * Enum implemented by score kind
 */
export enum ScoreKind {
  Primary = 'primary',
  Secondary = 'secondary',
}

/**
 * Enum implemented by checks
 */
export enum ReportOption {
  Adopters = 'adopters',
  ApprovedLicense = 'approved',
  ArtifactHubBadge = 'artifacthub_badge',
  Changelog = 'changelog',
  CodeOfConduct = 'code_of_conduct',
  CommunityMeeting = 'community_meeting',
  Contributing = 'contributing',
  DCO = 'dco',
  Governance = 'governance',
  LicenseScanning = 'scanning',
  Maintainers = 'maintainers',
  OpenSSFBadge = 'openssf_badge',
  Readme = 'readme',
  RecentRelease = 'recent_release',
  Roadmap = 'roadmap',
  SecurityPolicy = 'security_policy',
  SPDX = 'spdx_id',
  TrademarkFooter = 'trademark_footer',
  Website = 'website',
}

/**
 * Interface implemented by search filters query
 */
export interface SearchFiltersURL extends BasicQuery {
  pageNumber: number;
}

/**
 * Interface implemented as query base
 */
export interface BasicQuery {
  text?: string;
  filters?: {
    [key: string]: (string | number)[];
  };
}

/**
 * Interface implemented by search base
 */
export interface SearchQuery extends BasicQuery {
  limit: number;
  offset: number;
  sort_by: SortBy;
  sort_direction: SortDirection;
}

/**
 * Interface implemented by search data
 */
export interface SearchData {
  limit: number;
  offset: number;
  sort_by: string;
  sort_direction: string;
  text?: string;
  category?: number[];
  maturity?: number[];
  rating?: number[];
}

/**
 * Interface implemented by API error
 */
export interface Error {
  kind: ErrorKind;
  message?: string;
}

/**
 * Enum implemented by API error kind
 */
export enum ErrorKind {
  Other,
  NotFound,
}

/**
 * Interface implemented by check list info
 */
export type ReportOptionInfo = {
  [key in ReportOption]: ReportOptionData;
};
