'use client';

import React, { useRef, useEffect, useState, useCallback } from 'react';
import { Play, Pause, Loader2, AlertCircle } from 'lucide-react';

interface AudioPlayerProps {
  audioUrl: string;
  className?: string;
  isOwnMessage?: boolean;
}

export default function AudioPlayer({
                                      audioUrl,
                                      className = '',
                                      isOwnMessage = false
                                    }: AudioPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [duration, setDuration] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);
  const [waveformData, setWaveformData] = useState<number[]>([]);

  // Generate waveform data
  useEffect(() => {
    // Generate realistic waveform pattern
    const generateWaveform = () => {
      const data = [];
      for (let i = 0; i < 40; i++) {
        // Create a more natural waveform pattern
        const baseAmplitude = 0.3 + Math.random() * 0.4;
        const variation = Math.sin(i * 0.3) * 0.2;
        data.push(Math.max(0.1, Math.min(0.9, baseAmplitude + variation)));
      }
      return data;
    };

    setWaveformData(generateWaveform());
  }, [audioUrl]);

  // Draw waveform
  useEffect(() => {
    if (!canvasRef.current || waveformData.length === 0) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const { width, height } = canvas;
    const barWidth = width / waveformData.length;
    const progressWidth = duration > 0 ? (currentTime / duration) * width : 0;

    ctx.clearRect(0, 0, width, height);

    waveformData.forEach((amplitude, index) => {
      const barHeight = amplitude * height * 0.7;
      const x = index * barWidth;
      const y = (height - barHeight) / 2;

      // Set color based on progress and message type
      if (x < progressWidth) {
        ctx.fillStyle = isOwnMessage ? '#ffffff' : '#128C7E'; // Played portion
      } else {
        ctx.fillStyle = isOwnMessage ? 'rgba(255,255,255,0.4)' : 'rgba(18,140,126,0.3)'; // Unplayed portion
      }

      ctx.fillRect(x, y, Math.max(barWidth - 1, 1), barHeight);
    });
  }, [waveformData, currentTime, duration, isOwnMessage]);

  useEffect(() => {
    if (!audioUrl) return;

    const audio = audioRef.current;
    if (!audio) return;

    // Reset state
    setIsLoading(true);
    setError(null);
    setIsPlaying(false);
    setCurrentTime(0);
    setDuration(0);

    const handleLoadedMetadata = () => {
      console.log('Metadata loaded, duration:', audio.duration);
      if (audio.duration && isFinite(audio.duration)) {
        setDuration(audio.duration);
      }
    };

    const handleLoadedData = () => {
      console.log('Data loaded, duration:', audio.duration);
      setIsLoading(false);
      // Double check duration
      if (audio.duration && isFinite(audio.duration) && duration === 0) {
        setDuration(audio.duration);
      }
    };

    const handleCanPlayThrough = () => {
      console.log('Can play through, duration:', audio.duration);
      setIsLoading(false);
      // Final check for duration
      if (audio.duration && isFinite(audio.duration)) {
        setDuration(audio.duration);
      }
    };

    const handleDurationChange = () => {
      console.log('Duration changed:', audio.duration);
      if (audio.duration && isFinite(audio.duration)) {
        setDuration(audio.duration);
      }
    };

    const handleTimeUpdate = () => {
      setCurrentTime(audio.currentTime);
    };

    const handleEnded = () => {
      setIsPlaying(false);
      setCurrentTime(0);
    };

    const handleError = () => {
      setIsLoading(false);
        setError('Audio could not be loaded');
    };

    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);

    // Add event listeners
    audio.addEventListener('loadedmetadata', handleLoadedMetadata);
    audio.addEventListener('loadeddata', handleLoadedData);
    audio.addEventListener('canplaythrough', handleCanPlayThrough);
    audio.addEventListener('durationchange', handleDurationChange);
    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('ended', handleEnded);
    audio.addEventListener('error', handleError);
    audio.addEventListener('play', handlePlay);
    audio.addEventListener('pause', handlePause);

    // Set source and load
    audio.src = audioUrl;
    audio.load();

    // Cleanup
    return () => {
      audio.removeEventListener('loadedmetadata', handleLoadedMetadata);
      audio.removeEventListener('loadeddata', handleLoadedData);
      audio.removeEventListener('canplaythrough', handleCanPlayThrough);
      audio.removeEventListener('durationchange', handleDurationChange);
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('ended', handleEnded);
      audio.removeEventListener('error', handleError);
      audio.removeEventListener('play', handlePlay);
      audio.removeEventListener('pause', handlePause);
    };
  }, [audioUrl]);

  const togglePlay = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;

    if (isPlaying) {
      audio.pause();
    } else {
      audio.play().catch((e) => {
        console.error('Failed to play audio:', e);
        setError('Audio could not be played');
      });
    }
  }, [isPlaying]);

  const handleSeek = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const audio = audioRef.current;
    if (!audio || duration === 0) return;

    const canvas = e.currentTarget;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const newTime = percentage * duration;

    audio.currentTime = newTime;
    setCurrentTime(newTime);
  }, [duration]);

  const formatTime = (time: number) => {
    if (isNaN(time) || !isFinite(time) || time < 0) return '0:00';
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  if (error) {
    return (
        <div className={`flex items-center gap-2 p-3 max-w-xs ${
            isOwnMessage
                ? 'bg-red-500/10 border border-red-300'
                : 'bg-red-50 border border-red-200'
        } rounded-2xl ${className}`}>
          <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
          <span className="text-xs text-red-600 dark:text-red-400">{error}</span>
        </div>
    );
  }

  return (
      <div className={`flex items-center gap-3 p-3 max-w-xs ${
          isOwnMessage
              ? 'bg-[#005c4b] text-white'
              : 'bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700'
      } rounded-2xl shadow-sm ${className}`}>

        {/* Play/Pause Button */}
        <button
            onClick={togglePlay}
            disabled={isLoading}
            className={`flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center transition-all duration-200 ${
                isOwnMessage
                    ? 'bg-white/20 hover:bg-white/30 text-white'
                    : 'bg-[#128C7E] hover:bg-[#0d7267] text-white'
            } disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {isLoading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
          ) : isPlaying ? (
              <Pause className="w-4 h-4" />
          ) : (
              <Play className="w-4 h-4 ml-0.5" />
          )}
        </button>

        {/* Waveform and Info */}
        <div className="flex-1 min-w-0">
          {/* Waveform */}
          <canvas
              ref={canvasRef}
              width={160}
              height={32}
              className="w-full h-8 cursor-pointer mb-1"
              onClick={handleSeek}
          />

          {/* Time Info */}
          <div className="flex justify-between items-center">
          <span className={`text-xs font-medium ${
              isOwnMessage
                  ? 'text-white/80'
                  : 'text-slate-600 dark:text-slate-400'
          }`}>
            {formatTime(currentTime)}
          </span>

            {/* Total Duration - Always Show When Available */}
            <span className={`text-xs ${
                isOwnMessage
                    ? 'text-white/60'
                    : 'text-slate-500 dark:text-slate-500'
            }`}>
            {duration > 0 ? formatTime(duration) : (isLoading ? '...' : '0:00')}
          </span>
          </div>
        </div>

        {/* Voice Message Icon */}
        <div className={`flex-shrink-0 ${
            isOwnMessage
                ? 'text-white/60'
                : 'text-slate-400 dark:text-slate-500'
        }`}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 1c-4.97 0-9 4.03-9 9v7c0 1.66 1.34 3 3 3h3v-8H5v-2c0-3.87 3.13-7 7-7s7 3.13 7 7v2h-4v8h3c1.66 0 3-1.34 3-3v-7c0-4.97-4.03-9-9-9z"/>
          </svg>
        </div>

        {/* Hidden audio element */}
        <audio ref={audioRef} preload="auto" />
      </div>
  );
}