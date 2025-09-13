import React, { useState } from 'react';
import AudioPlayer from '@/components/chat/AudioPlayer';
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';

interface MessageContentProps {
    content: string;
}

// Regex patterns
const imageMarkdownRegex = /\[image\]\((.*?)\)/;
const audioMarkdownRegex = /\[audio\]\((.*?)\)/;
const videoMarkdownRegex = /\[video\]\((.*?)\)/; // Supports [video](url) markdown
const urlRegex = /(https?:\/\/[^\s"'<>()]+)/g; // Match links inside the text
const youtubeRegex = /(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/(?:[^\/\n\s]+\/\S+\/|(?:v|e(?:mbed)?)\/|\S*?[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;

/**
 * Extracts YouTube video ID from various YouTube URL formats
 */
const getYouTubeVideoId = (url: string): string | null => {
    const match = url.match(youtubeRegex);
    return match ? match[1] : null;
};

/**
 * Checks if a URL is a YouTube URL
 */
const isYouTubeUrl = (url: string): boolean => {
    return youtubeRegex.test(url);
};

/**
 * YouTube Player Component
 */
const YouTubePlayer: React.FC<{ videoId: string; url: string }> = ({ videoId, url }) => {
    const [isLoading, setIsLoading] = useState(true);
    const [hasError, setHasError] = useState(false);

    const embedUrl = `https://www.youtube.com/embed/${videoId}`;

    if (hasError) {
        return (
            <div className="flex items-center gap-3 p-4 bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 rounded-xl border border-dashed border-red-300 dark:border-red-600 shadow-sm">
                <div className="text-red-500 dark:text-red-400">
                    <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                </div>
                <div className="text-sm text-red-600 dark:text-red-300">
                    <div className="font-semibold">YouTube videosu yüklenemedi</div>
                    <a
                        href={url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-xs underline hover:no-underline"
                        onClick={(e) => e.stopPropagation()}
                    >
                        Videoyu YouTube'da aç
                    </a>
                </div>
            </div>
        );
    }

    return (
        <div className="mt-1">
            <div className="relative overflow-hidden rounded-xl shadow-lg bg-gray-900">
                {isLoading && (
                    <div className="absolute inset-0 flex items-center justify-center bg-gray-100 dark:bg-gray-800 animate-pulse">
                        <div className="text-gray-500 dark:text-gray-400">
                            <svg className="w-12 h-12 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                        </div>
                    </div>
                )}
                <iframe
                    src={embedUrl}
                    title={`YouTube video ${videoId}`}
                    className="w-full aspect-video max-w-[400px] md:max-w-md"
                    frameBorder="0"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                    allowFullScreen
                    onLoad={() => setIsLoading(false)}
                    onError={() => {
                        setIsLoading(false);
                        setHasError(true);
                    }}
                />
            </div>
            <div className="mt-2 text-xs text-gray-500 dark:text-gray-400">
                <a
                    href={url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:text-blue-400 transition-colors duration-200"
                    onClick={(e) => e.stopPropagation()}
                >
                    YouTube'da aç →
                </a>
            </div>
        </div>
    );
};

/**
 * Parses the message content and renders it as an image, video, link, or plain text.
 */
export const MessageContent: React.FC<MessageContentProps> = ({ content }) => {
    const [imageError, setImageError] = useState(false);

    // 1. Check if the content is an audio file
    const audioMatch = content.match(audioMarkdownRegex);
    if (audioMatch && audioMatch[1]) {
        const audioUrl = audioMatch[1];
        console.log('Audio message found:', audioUrl);
        return (
            <div className="mt-1">
                <AudioPlayer audioUrl={audioUrl} />
            </div>
        );
    }

    // 2. Check if the content is a video
    const videoMatch = content.match(videoMarkdownRegex);
    if (videoMatch && videoMatch[1]) {
        const videoUrl = videoMatch[1];
        return (
            <div className="mt-1">
                <video
                    src={videoUrl}
                    controls
                    className="max-w-[280px] md:max-w-xs w-full h-auto rounded-xl shadow-lg"
                />
            </div>
        );
    }

    // 3. Check if the content is an image
    const imageMatch = content.match(imageMarkdownRegex);
    if (imageMatch && imageMatch[1]) {
        const imageUrl = imageMatch[1];
        return (
            <div className="mt-1">
                {!imageError ? (
                    <Dialog>
                        <DialogTrigger asChild>
                            <button
                                type="button"
                                className="block group"
                                onClick={(e) => e.stopPropagation()} // Prevent triggering parent click handlers
                                aria-label="Zoom image"
                            >
                                <div className="relative overflow-hidden rounded-xl shadow-lg transition-all duration-300 hover:shadow-xl">
                                    <img
                                        src={imageUrl}
                                        alt="Image uploaded to chat"
                                        className="max-w-[280px] md:max-w-xs w-full h-auto object-cover transition-transform duration-300 group-hover:scale-105"
                                        onError={() => setImageError(true)}
                                        onLoad={() => setImageError(false)}
                                    />
                                    <div className="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300" />
                                </div>
                            </button>
                        </DialogTrigger>
                        <DialogContent className="max-w-3xl w-[90vw] md:w-auto p-0 bg-transparent border-none shadow-none">
                            <div className="relative">
                                <img
                                    src={imageUrl}
                                    alt="Image uploaded in chat (expanded)"
                                    className="w-full h-auto rounded-xl"
                                />
                            </div>
                        </DialogContent>
                    </Dialog>
                ) : (
                    <div className="flex items-center gap-3 p-4 bg-gradient-to-r from-gray-100 to-gray-200 dark:from-gray-800 dark:to-gray-700 rounded-xl border border-dashed border-gray-300 dark:border-gray-600 shadow-sm">
                        <div className="text-gray-500 dark:text-gray-400">
                            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                            </svg>
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-300">
                            <div className="font-semibold">Image not available</div>
                            <div className="text-xs opacity-75">The image could not be loaded</div>
                        </div>
                    </div>
                )}
            </div>
        );
    }

    // 4. Check if the content is a single YouTube URL
    const trimmedContent = content.trim();
    if (isYouTubeUrl(trimmedContent)) {
        const videoId = getYouTubeVideoId(trimmedContent);
        if (videoId) {
            return <YouTubePlayer videoId={videoId} url={trimmedContent} />;
        }
    }

    // 5. If plain text, parse and linkify URLs (including YouTube URLs in text)
    const parts = content.split(urlRegex);

    return (
        <div className="space-y-3">
            <div className="whitespace-pre-wrap break-words leading-relaxed">
                {parts.map((part, index) => {
                    // If the part matches the URL regex, check if it's a YouTube URL
                    if (part.match(urlRegex)) {
                        if (isYouTubeUrl(part)) {
                            const videoId = getYouTubeVideoId(part);
                            if (videoId) {
                                return (
                                    <div key={index} className="my-3">
                                        <YouTubePlayer videoId={videoId} url={part} />
                                    </div>
                                );
                            }
                        }

                        // Regular link
                        return (
                            <a
                                key={index}
                                href={part}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-blue-400 hover:text-blue-300 underline decoration-blue-400/30 hover:decoration-blue-300/50 transition-all duration-200 break-all"
                                onClick={(e) => e.stopPropagation()} // Prevent triggering parent click handlers
                            >
                                {part}
                            </a>
                        );
                    }
                    // Non-matching parts are plain text
                    return <React.Fragment key={index}>{part}</React.Fragment>;
                })}
            </div>
        </div>
    );
};