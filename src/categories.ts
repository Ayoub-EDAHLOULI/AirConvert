import {
  FileImage,
  Music,
  FileText,
  Sheet,
  Video,
  type LucideIcon,
} from "lucide-react";

export interface CategoryDef {
  path: string;
  label: string;
  description: string;
  icon: LucideIcon;
  formats: string;
  available: boolean;
}

export const CATEGORIES: CategoryDef[] = [
  {
    path: "/images",
    label: "Images",
    description: "Convert between common image formats",
    icon: FileImage,
    formats: "jpg, png, webp, gif, bmp, tiff, svg, ico, tga, pnm, qoi, avif",
    available: true,
  },
  {
    path: "/audio",
    label: "Audio",
    description: "Convert between common audio formats",
    icon: Music,
    formats: "mp3, wav, flac, ogg, m4a, aac, opus, wma",
    available: true,
  },
  {
    path: "/documents",
    label: "Documents",
    description: "Convert between document and markup formats",
    icon: FileText,
    formats: "md, txt, html, rtf, odt, docx",
    available: true,
  },
  {
    path: "/spreadsheets",
    label: "Spreadsheets",
    description: "Convert between spreadsheet and data formats",
    icon: Sheet,
    formats: "csv, xlsx, xls, ods (ods input only)",
    available: true,
  },
  {
    path: "/video",
    label: "Video",
    description: "Convert between common video formats",
    icon: Video,
    formats: "mp4, mov, avi, webm, gif",
    available: false,
  },
];
