import axios from 'axios';
import { JSDOM } from 'jsdom';

export interface WebScraperOptions {
  timeout?: number;
  userAgent?: string;
  maxDepth?: number;
  maxPages?: number;
  onError?: (error: Error) => void;
  headers?: Record<string, string>;
  followRedirects?: boolean;
}

export interface ScrapedDocument {
  url: string;
  title: string;
  content: string;
  html: string;
  links: string[];
  metadata: Record<string, string>;
  timestamp: string;
}

/**
 * WebScraper class for extracting content from web pages
 * Used for documentation scraping and RAG context building
 */
class WebScraper {
  private options: Required<WebScraperOptions>;
  private visitedUrls: Set<string>;

  constructor(options: WebScraperOptions = {}) {
    this.options = {
      timeout: 30000,
      userAgent: 'Auto-Coder-Companion/1.0',
      maxDepth: 1,
      maxPages: 10,
      onError: (error: Error) => console.error('WebScraper error:', error),
      headers: {},
      followRedirects: true,
      ...options
    };
    this.visitedUrls = new Set<string>();
  }

  /**
   * Scrape a single URL and extract content
   * @param url The URL to scrape
   * @returns Scraped document with content and metadata
   */
  async scrapeUrl(url: string): Promise<ScrapedDocument> {
    try {
      const response = await axios.get(url, {
        timeout: this.options.timeout,
        headers: {
          'User-Agent': this.options.userAgent,
          ...this.options.headers
        },
        maxRedirects: this.options.followRedirects ? 5 : 0
      });

      const html = response.data;
      const dom = new JSDOM(html);
      const document = dom.window.document;

      // Extract title
      const title = document.querySelector('title')?.textContent || 'Untitled Document';

      // Extract content
      const content = this.extractTextContent(document);

      // Extract links
      const links = Array.from(document.querySelectorAll('a'))
        .map(a => a.getAttribute('href'))
        .filter((href): href is string => {
          if (href === null) return false;
          return !href.startsWith('#') && !href.startsWith('javascript:');
        });

      // Extract metadata
      const metadata: Record<string, string> = {};
      const metaTags = document.querySelectorAll('meta');
      metaTags.forEach(meta => {
        const name = meta.getAttribute('name') || meta.getAttribute('property');
        const content = meta.getAttribute('content');
        if (name && content) {
          metadata[name] = content;
        }
      });

      return {
        url,
        title,
        content,
        html,
        links,
        metadata,
        timestamp: new Date().toISOString()
      };
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.options.onError(error);
      throw error;
    }
  }

  /**
   * Recursively scrape URLs starting from a root URL
   * @param startUrl The starting URL
   * @param depth Current depth (used internally for recursion)
   * @returns Array of scraped documents
   */
  async scrapeUrlsRecursively(startUrl: string, depth: number = 0): Promise<ScrapedDocument[]> {
    if (depth > this.options.maxDepth) {
      return [];
    }

    this.visitedUrls = new Set<string>();
    const queue: { url: string; depth: number }[] = [{ url: startUrl, depth }];
    const results: ScrapedDocument[] = [];

    // Use AbortController for timeout handling
    const controller = new AbortController();
    const timeoutId = setTimeout(() => {
      controller.abort();
    }, this.options.timeout * 2); // Double the regular timeout for the entire operation

    try {
      while (queue.length > 0 && results.length < this.options.maxPages) {
        const { url, depth } = queue.shift()!;

        if (this.visitedUrls.has(url)) {
          continue;
        }

        this.visitedUrls.add(url);

        try {
          const document = await this.scrapeUrl(url);
          results.push(document);

          if (depth < this.options.maxDepth) {
            // Add links to queue
            for (const link of document.links) {
              try {
                const absoluteUrl = new URL(link, url).href;
                if (!this.visitedUrls.has(absoluteUrl)) {
                  queue.push({ url: absoluteUrl, depth: depth + 1 });
                }
              } catch (e) {
                // Invalid URL, skip
              }
            }
          }
        } catch (err) {
          // Skip failed URLs but log the error
          console.error(`Failed to scrape ${url}:`, err);
        }
      }

      return results;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Extract clean text content from a document
   * @param document DOM document
   * @returns Cleaned text content
   */
  private extractTextContent(document: Document): string {
    // Create a clone to avoid modifying the original
    const clone = document.cloneNode(true) as Document;
    
    // Remove script, style, and other non-content elements
    const elementsToRemove = clone.querySelectorAll('script, style, noscript, iframe, svg, canvas, video, audio');
    elementsToRemove.forEach(el => el.remove());

    // Get body content
    const body = clone.querySelector('body');
    if (!body) {
      return '';
    }

    // Extract text content
    return this.cleanText(body.textContent || '');
  }

  /**
   * Clean and normalize text content
   * @param text Raw text
   * @returns Cleaned text
   */
  private cleanText(text: string): string {
    return text
      .replace(/\s+/g, ' ')      // Replace multiple spaces with a single space
      .replace(/\n+/g, '\n')     // Replace multiple newlines with a single newline
      .replace(/\t+/g, ' ')      // Replace tabs with spaces
      .trim();                   // Remove leading/trailing whitespace
  }
}

export default WebScraper;
