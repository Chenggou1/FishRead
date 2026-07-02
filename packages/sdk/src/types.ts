export const PROTOCOL_VERSION = 1;

export interface ApiResponse<T> {
  protocol_version: typeof PROTOCOL_VERSION;
  ok: true;
  data: T;
}

export interface ApiError {
  protocol_version: typeof PROTOCOL_VERSION;
  ok: false;
  error: { code: string; message: string };
}

export type ApiResult<T> = ApiResponse<T> | ApiError;

export interface BookDto {
  id: string;
  title: string;
  author?: string;
  format: string;
}

export interface BookListItemDto {
  id: string;
  title: string;
  author?: string;
  format: string;
  current: boolean;
  imported_at: number;
  position: PositionDto;
  reading_anchor_label: string;
}

export interface BookListDto {
  books: BookListItemDto[];
}

export interface BookUseDto {
  book: BookDto;
  position: PositionDto;
}

export interface BookDeleteDto {
  deleted: BookDto;
  cleared_current: boolean;
}

export interface BookRefDto {
  id: string;
  title: string;
}

export interface PositionDto {
  chapter_index: number;
  chunk_index: number;
}

export interface ChapterRefDto {
  id: string;
  index: number;
  title: string;
}

export interface ChunkDto {
  index: number;
  text: string;
  is_first: boolean;
  is_last: boolean;
}

export interface ProgressDto {
  chapter_index: number;
  chunk_index: number;
  chapter_percent: number;
  book_percent: number;
}

export interface ReaderStateDto {
  book: BookDto;
  chapter: ChapterRefDto;
  chunk: ChunkDto;
  progress: ProgressDto;
  start_of_book: boolean;
  end_of_book: boolean;
}

export interface AnchorChunkDto {
  index: number;
  is_first: boolean;
  is_last: boolean;
}

export interface ReadingAnchorDto {
  label: string;
  chapter_percent: number;
  current: boolean;
  position: PositionDto;
  chunk: AnchorChunkDto;
  preview: string;
}

export interface ChapterListItemDto {
  id: string;
  index: number;
  title: string;
  current: boolean;
  anchors?: ReadingAnchorDto[];
}

export interface ChapterListDto {
  book: BookRefDto;
  chapters: ChapterListItemDto[];
}
