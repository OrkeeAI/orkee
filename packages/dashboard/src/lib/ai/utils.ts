// ABOUTME: Utility functions for AI operations including token estimation and text chunking
// ABOUTME: Provides helpers for size validation, cost calculation, and intelligent text splitting

/**
 * Estimate token count from text
 * Uses rough approximation: ~4 characters per token for English text
 */
export function estimateTokens(text: string): number {
  // More accurate estimation accounting for whitespace and punctuation
  return Math.ceil(text.length / 3.5);
}

/**
 * Check if content size is within safe limits
 */
export function validateContentSize(
  content: string,
  maxTokens: number,
  promptOverhead = 500
): { valid: boolean; estimatedTokens: number; reason?: string } {
  const estimatedTokens = estimateTokens(content);
  const totalTokens = estimatedTokens + promptOverhead;

  if (totalTokens > maxTokens) {
    return {
      valid: false,
      estimatedTokens,
      reason: `Content estimated at ${estimatedTokens} tokens (+ ${promptOverhead} prompt overhead) exceeds limit of ${maxTokens} tokens`,
    };
  }

  return { valid: true, estimatedTokens };
}

/**
 * Split text into semantic chunks
 * Tries to break at natural boundaries (paragraphs, sentences, or fixed size)
 */
export function chunkText(text: string, maxChunkTokens: number): string[] {
  const chunks: string[] = [];
  const targetCharsPerChunk = Math.floor(maxChunkTokens * 3.5);

  // If text is small enough, return as-is
  if (text.length <= targetCharsPerChunk) {
    return [text];
  }

  // First, try to split by double newlines (paragraphs)
  const paragraphs = text.split(/\n\n+/);

  let currentChunk = '';

  for (const paragraph of paragraphs) {
    const paragraphWithNewlines = paragraph + '\n\n';

    // If single paragraph exceeds chunk size, split by sentences
    if (paragraph.length > targetCharsPerChunk) {
      // Save current chunk if exists
      if (currentChunk.trim()) {
        chunks.push(currentChunk.trim());
        currentChunk = '';
      }

      // Split large paragraph by sentences
      const sentences = paragraph.split(/(?<=[.!?])\s+/);

      // If no sentence boundaries, split by fixed size
      if (sentences.length === 1 && sentences[0].length > targetCharsPerChunk) {
        // Hard split at character boundary
        let remaining = paragraph;
        while (remaining.length > targetCharsPerChunk) {
          chunks.push(remaining.substring(0, targetCharsPerChunk));
          remaining = remaining.substring(targetCharsPerChunk);
        }
        if (remaining.trim()) {
          currentChunk = remaining + ' ';
        }
      } else {
        for (const sentence of sentences) {
          if (currentChunk.length + sentence.length + 1 > targetCharsPerChunk) {
            if (currentChunk.trim()) {
              chunks.push(currentChunk.trim());
            }
            currentChunk = sentence + ' ';
          } else {
            currentChunk += sentence + ' ';
          }
        }
      }
    } else if (currentChunk.length + paragraphWithNewlines.length > targetCharsPerChunk) {
      // Current paragraph would exceed chunk size, save current and start new
      if (currentChunk.trim()) {
        chunks.push(currentChunk.trim());
      }
      currentChunk = paragraphWithNewlines;
    } else {
      // Add paragraph to current chunk
      currentChunk += paragraphWithNewlines;
    }
  }

  // Don't forget the last chunk
  if (currentChunk.trim()) {
    chunks.push(currentChunk.trim());
  }

  return chunks.length > 0 ? chunks : [text];
}

/**
 * Create a summary prompt for chunk context
 */
export function createChunkPrompt(
  chunk: string,
  chunkIndex: number,
  totalChunks: number,
  basePrompt: string
): string {
  if (totalChunks === 1) {
    return basePrompt + '\n\n' + chunk;
  }

  return `${basePrompt}

NOTE: This is part ${chunkIndex + 1} of ${totalChunks} of a larger PRD. Focus on extracting capabilities and requirements from this section. You will be able to reference other sections later.

PRD Section ${chunkIndex + 1}/${totalChunks}:
${chunk}`;
}

/**
 * Estimate cost for processing text
 */
export function estimateProcessingCost(
  text: string,
  inputCostPer1k: number,
  outputCostPer1k: number,
  estimatedOutputTokens: number
): {
  inputTokens: number;
  outputTokens: number;
  totalCost: number;
} {
  const inputTokens = estimateTokens(text);
  const outputTokens = estimatedOutputTokens;

  const inputCost = (inputTokens / 1000) * inputCostPer1k;
  const outputCost = (outputTokens / 1000) * outputCostPer1k;

  return {
    inputTokens,
    outputTokens,
    totalCost: inputCost + outputCost,
  };
}

/**
 * Create timeout promise for race conditions
 */
export function createTimeout<T>(ms: number, errorMessage: string): Promise<T> {
  return new Promise((_, reject) => {
    setTimeout(() => {
      reject(new Error(errorMessage));
    }, ms);
  });
}

/**
 * Wrap a promise with timeout
 */
export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  operation: string
): Promise<T> {
  return Promise.race([promise, createTimeout<T>(timeoutMs, `${operation} timed out after ${timeoutMs}ms`)]);
}
