import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export const MarkdownRender: React.FC<{
  content: string;
}> = ({ content }) => {
  return (
    <ReactMarkdown remarkPlugins={[remarkGfm]} className="markdown-container">
      {content}
    </ReactMarkdown>
  );
};
