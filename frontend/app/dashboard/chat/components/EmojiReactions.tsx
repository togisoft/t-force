import React from 'react';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Button } from '@/components/ui/button';
import { useReactions } from '../hooks/useReactions';

interface EmojiReactionsProps {
  messageId: string;
  onAddReaction: (messageId: string, emoji: string) => void;
  trigger: React.ReactNode;
}

export function EmojiReactions({ messageId, onAddReaction, trigger }: EmojiReactionsProps) {
  const { availableEmojis } = useReactions();
  const [isOpen, setIsOpen] = React.useState(false);

  const handleEmojiClick = (emoji: string) => {
    onAddReaction(messageId, emoji);
    setIsOpen(false);
  };

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        {trigger}
      </PopoverTrigger>
      <PopoverContent className="w-auto p-2" align="end">
        <div className="flex flex-wrap gap-1 max-w-[220px]">
          {availableEmojis.map((emojiData) => (
            <Button
              key={emojiData.emoji}
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0"
              onClick={() => handleEmojiClick(emojiData.emoji)}
              title={emojiData.label}
            >
              <span className="text-lg">{emojiData.emoji}</span>
            </Button>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}