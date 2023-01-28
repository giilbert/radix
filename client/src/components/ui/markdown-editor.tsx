import {
  Box,
  HStack,
  Text,
  SimpleGrid,
  Textarea,
  TextareaProps,
} from "@chakra-ui/react";
import { forwardRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export const MarkdownEditor = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ onChange, ...rest }, ref) => {
    const [value, setValue] = useState(rest.defaultValue as string);

    return (
      <SimpleGrid gridTemplateColumns="1fr 1fr" gap="2">
        <Textarea
          {...rest}
          onChange={(e) => setValue(e.target.value)}
          fontFamily="mono"
          ref={ref}
        />

        <Box
          p="4"
          borderColor="whiteAlpha.300"
          borderStyle="solid"
          borderWidth="1px"
          borderRadius="md"
        >
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            className="markdown-container"
          >
            {value === "" ? "Preview will go here.." : value}
          </ReactMarkdown>
        </Box>
      </SimpleGrid>
    );
  }
);

MarkdownEditor.displayName = "MarkdownEditor";
