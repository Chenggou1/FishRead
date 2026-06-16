// CLI JSON Protocol DTOs — mirrors fishread-core/src/protocol/dto.rs

export interface ApiResponse<T> {
  ok: true;
  data: T;
}

export interface ApiError {
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
