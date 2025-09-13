'use client';

import React, { useState, useRef, useEffect } from 'react';
import { Mic, MicOff, Play, Pause, Send, X, Volume2, Square, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';

interface VoiceMessageProps {
  onSend: (audioBlob: Blob) => void;
  isRecording?: boolean;
  disabled?: boolean;
}

export const VoiceMessage: React.FC<VoiceMessageProps> = ({
  onSend,
  isRecording = false,
  disabled = false,
}) => {
  const [isRecordingLocal, setIsRecordingLocal] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [audioBlob, setAudioBlob] = useState<Blob | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [duration, setDuration] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);
  const [recordingTime, setRecordingTime] = useState(0);
  const [recordingStartTime, setRecordingStartTime] = useState<number | null>(null);
  const [pausedTime, setPausedTime] = useState(0);
  const [isPopoverOpen, setIsPopoverOpen] = useState(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const streamRef = useRef<MediaStream | null>(null);

  const startRecording = async () => {
    console.log('Starting recording...');
    try {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        throw new Error('Media devices not supported');
      }

      const stream = await navigator.mediaDevices.getUserMedia({ 
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
          sampleRate: 44100
        } 
      });
      
      streamRef.current = stream;
      
      const mimeType = [
        'audio/webm;codecs=opus',
        'audio/webm',
        'audio/mp4',
        'audio/ogg;codecs=opus',
        'audio/wav'
      ].find(type => MediaRecorder.isTypeSupported(type)) || 'audio/webm';
      
      console.log('Using MIME type:', mimeType);
      
      const mediaRecorder = new MediaRecorder(stream, { mimeType });
      mediaRecorderRef.current = mediaRecorder;
      chunksRef.current = [];

      mediaRecorder.ondataavailable = (event) => {
        console.log('Data available:', event.data.size, 'bytes');
        if (event.data.size > 0) {
          chunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = () => {
        console.log('Recording stopped, chunks:', chunksRef.current.length);
        if (chunksRef.current.length > 0) {
          const blob = new Blob(chunksRef.current, { 
            type: mediaRecorder.mimeType || 'audio/webm' 
          });
          console.log('Created blob:', blob.size, 'bytes');
          setAudioBlob(blob);
          const url = URL.createObjectURL(blob);
          setAudioUrl(url);
          // Keep popover open for playback
        } else {
          alert('No audio was recorded. Please try again.');
          setIsPopoverOpen(false);
        }
        if (streamRef.current) {
          streamRef.current.getTracks().forEach(track => track.stop());
          streamRef.current = null;
        }
        setRecordingTime(0);
        setRecordingStartTime(null);
        setPausedTime(0);
        setIsPaused(false);
        setIsRecordingLocal(false);
      };

      mediaRecorder.onerror = (event) => {
        console.error('MediaRecorder error:', event);
        if (streamRef.current) {
          streamRef.current.getTracks().forEach(track => track.stop());
          streamRef.current = null;
        }
        setIsRecordingLocal(false);
        setRecordingTime(0);
        setRecordingStartTime(null);
        setPausedTime(0);
        setIsPaused(false);
        setIsPopoverOpen(false);
      };

      mediaRecorder.start(100);
      setIsRecordingLocal(true);
      setRecordingStartTime(Date.now());
      setRecordingTime(0);
      setPausedTime(0);
      setIsPaused(false);
      setIsPopoverOpen(true);
      console.log('Recording started successfully');
      
    } catch (error) {
      console.error('Error starting recording:', error);
      alert('Failed to start recording. Please check microphone permissions.');
    }
  };

  const pauseRecording = () => {
    console.log('Pausing recording...');
    if (mediaRecorderRef.current && isRecordingLocal && !isPaused) {
      mediaRecorderRef.current.pause();
      setIsPaused(true);
      setPausedTime(recordingTime);
      console.log('Recording paused');
    }
  };

  const resumeRecording = () => {
    console.log('Resuming recording...');
    if (mediaRecorderRef.current && isRecordingLocal && isPaused) {
      mediaRecorderRef.current.resume();
      setIsPaused(false);
      setRecordingStartTime(Date.now() - (pausedTime * 1000));
      console.log('Recording resumed');
    }
  };

  const stopRecording = () => {
    console.log('Stopping recording...');
    if (mediaRecorderRef.current && isRecordingLocal) {
      mediaRecorderRef.current.stop();
    }
  };

  const playAudio = async () => {
    console.log('Playing audio...', audioRef.current, audioUrl);
    if (audioRef.current && audioUrl) {
      try {
        if (isPlaying) {
          audioRef.current.pause();
          setIsPlaying(false);
        } else {
          // Force reload audio to ensure metadata is loaded
          audioRef.current.load();
          await audioRef.current.play();
          setIsPlaying(true);
        }
      } catch (error) {
        console.error('Error playing audio:', error);
      }
    }
  };

  const handleAudioTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime);
    }
  };

  const handleAudioLoadedMetadata = () => {
    if (audioRef.current) {
      const duration = audioRef.current.duration;
      console.log('Audio duration loaded:', duration);
      if (duration && isFinite(duration)) {
        setDuration(duration);
      }
    }
  };

  const handleAudioCanPlay = () => {
    if (audioRef.current) {
      const duration = audioRef.current.duration;
      console.log('Audio can play, duration:', duration);
      if (duration && isFinite(duration)) {
        setDuration(duration);
      }
    }
  };

  const handleAudioEnded = () => {
    setIsPlaying(false);
    setCurrentTime(0);
  };

  const handleAudioError = (e: React.SyntheticEvent<HTMLAudioElement, Event>) => {
    console.error('Audio error in VoiceMessage:', e);
  };

  const handleSend = () => {
    if (audioBlob) {
      onSend(audioBlob);
      setAudioBlob(null);
      setCurrentTime(0);
      setDuration(0);
      setIsPopoverOpen(false);
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }
    }
  };

  const handleCancel = () => {
    setAudioBlob(null);
    setCurrentTime(0);
    setDuration(0);
    setIsPlaying(false);
    setRecordingTime(0);
    setRecordingStartTime(null);
    setPausedTime(0);
    setIsPaused(false);
    setIsPopoverOpen(false);
    if (audioUrl) {
      URL.revokeObjectURL(audioUrl);
      setAudioUrl(null);
    }
  };

  const handleRerecord = () => {
    setAudioBlob(null);
    setCurrentTime(0);
    setDuration(0);
    setIsPlaying(false);
    setRecordingTime(0);
    setRecordingStartTime(null);
    setPausedTime(0);
    setIsPaused(false);
    if (audioUrl) {
      URL.revokeObjectURL(audioUrl);
      setAudioUrl(null);
    }
    // Start recording again
    startRecording();
  };

  // Update recording time while recording
  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (isRecordingLocal && recordingStartTime && !isPaused) {
      interval = setInterval(() => {
        const elapsed = (Date.now() - recordingStartTime) / 1000;
        setRecordingTime(elapsed + pausedTime);
      }, 100);
    }
    
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [isRecordingLocal, recordingStartTime, isPaused, pausedTime]);

  const formatTime = (time: number) => {
    if (!isFinite(time) || time < 0) {
      return '0:00';
    }
    
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  return (
    <div className="w-full">
      <Popover open={isPopoverOpen} onOpenChange={setIsPopoverOpen}>
        <PopoverTrigger asChild>
          <Button
            onClick={startRecording}
            disabled={disabled}
            data-voice-record
            className="absolute -top-1000 left-0 opacity-0 pointer-events-none"
          >
            Record
          </Button>
        </PopoverTrigger>
        <PopoverContent 
          className="w-80 p-0 border-0 shadow-2xl" 
          align="end"
          side="top"
          sideOffset={10}
        >
          {/* Recording Interface */}
          {isRecordingLocal && (
            <div className="bg-white rounded-2xl p-4 shadow-xl">
              <div className="text-center">
                <div className="mb-3">
                  <div className={`w-16 h-16 rounded-full flex items-center justify-center mx-auto ${
                    isPaused ? "bg-yellow-500" : "bg-red-500"
                  } animate-pulse`}>
                    {isPaused ? (
                      <Pause className="h-6 w-6 text-white" />
                    ) : (
                      <MicOff className="h-6 w-6 text-white" />
                    )}
                  </div>
                </div>
                
                <h3 className="text-sm font-semibold text-gray-900 mb-2">
                  {isPaused ? 'Recording Paused' : 'Recording...'}
                </h3>
                
                <p className="text-xl font-mono text-red-500 mb-4">
                  {formatTime(recordingTime)}
                </p>
                
                {/* Recording Control Buttons */}
                <div className="flex gap-2 mb-3">
                  {isPaused ? (
                    <button
                      onClick={resumeRecording}
                      className="flex-1 bg-yellow-500 hover:bg-yellow-600 text-white py-2 px-3 rounded-lg text-sm font-medium transition-colors"
                    >
                      Continue
                    </button>
                  ) : (
                    <button
                      onClick={pauseRecording}
                      className="flex-1 bg-yellow-500 hover:bg-yellow-600 text-white py-2 px-3 rounded-lg text-sm font-medium transition-colors"
                    >
                      Pause
                    </button>
                  )}
                  
                  <button
                    onClick={stopRecording}
                    className="flex-1 bg-red-500 hover:bg-red-600 text-white py-2 px-3 rounded-lg text-sm font-medium transition-colors"
                  >
                    Stop
                  </button>
                </div>
                
                <button
                  onClick={handleCancel}
                  className="w-full border border-gray-300 hover:bg-gray-50 text-gray-600 py-2 rounded-lg text-sm transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {/* Playback Interface */}
          {audioBlob && !isRecordingLocal && (
            <div className="bg-white rounded-2xl p-4 shadow-xl">
              <div className="text-center">
                <div className="mb-3">
                  <div className="w-16 h-16 bg-blue-500 rounded-full flex items-center justify-center mx-auto">
                    <Volume2 className="h-6 w-6 text-white" />
                  </div>
                </div>
                
                <h3 className="text-sm font-semibold text-gray-900 mb-3">
                  Voice Message
                </h3>
                
                <div className="mb-4">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-xs font-mono text-gray-500">
                      {formatTime(currentTime)}
                    </span>
                    
                    <div className="flex-1 bg-gray-200 rounded-full h-1.5 overflow-hidden">
                      <div
                        className="bg-blue-500 h-full rounded-full transition-all duration-300 ease-out"
                        style={{
                          width: duration > 0 ? `${(currentTime / duration) * 100}%` : '0%'
                        }}
                      />
                    </div>
                    
                    <span className="text-xs font-mono text-gray-500">
                      {formatTime(duration)}
                    </span>
                  </div>
                </div>
                
                {/* Playback Control Buttons */}
                <div className="flex gap-2 mb-3">
                  <button
                    onClick={playAudio}
                    className="flex-1 bg-blue-500 hover:bg-blue-600 text-white py-2 px-3 rounded-lg text-sm font-medium transition-colors"
                  >
                    {isPlaying ? 'Pause' : 'Listen'}
                  </button>
                  
                  <button
                    onClick={handleSend}
                    className="flex-1 bg-green-500 hover:bg-green-600 text-white py-2 px-3 rounded-lg text-sm font-medium transition-colors"
                  >
                    Send
                  </button>
                </div>
                
                <div className="flex gap-2">
                  <button
                    onClick={handleRerecord}
                    className="flex-1 border border-orange-300 hover:bg-orange-50 text-orange-600 py-2 rounded-lg text-sm transition-colors"
                  >
                    Re-record
                  </button>
                  
                  <button
                    onClick={handleCancel}
                    className="flex-1 border border-red-300 hover:bg-red-50 text-red-500 py-2 rounded-lg text-sm transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          )}
        </PopoverContent>
      </Popover>

      {audioUrl && (
        <audio
          ref={audioRef}
          src={audioUrl}
          onTimeUpdate={handleAudioTimeUpdate}
          onLoadedMetadata={handleAudioLoadedMetadata}
          onCanPlay={handleAudioCanPlay}
          onEnded={handleAudioEnded}
          onError={handleAudioError}
          preload="metadata"
          className="hidden"
        />
      )}
    </div>
  );
}; 