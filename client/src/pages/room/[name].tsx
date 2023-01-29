import { Chat } from "@/components/rooms/chat";
import { Editors } from "@/components/rooms/editors";
import { RoomProvider } from "@/components/rooms/room-provider";
import { SidePanel } from "@/components/rooms/side-panel";
import { TopBar } from "@/components/rooms/top-bar";
import { Authed } from "@/components/ui/authed";
import { useIsMobile } from "@/lib/hooks/use-is-mobile";
import {
  Box,
  Button,
  Code,
  GridItem,
  Heading,
  HStack,
  IconButton,
  SimpleGrid,
  Text,
} from "@chakra-ui/react";
import { NextPage } from "next";
import { useRouter } from "next/router";
import { FiMessageSquare } from "react-icons/fi";
import { MdOutlineDescription } from "react-icons/md";

const RoomPage: NextPage = () => {
  const isMobile = useIsMobile();
  const router = useRouter();

  const layout = isMobile ? (
    <Box overflowX="hidden" w="100vw">
      <TopBar />
      <Editors />
    </Box>
  ) : (
    <SimpleGrid gridTemplateColumns={["4fr 6fr 3fr"]} h="100vh">
      <GridItem bg="blackAlpha.300">
        <SidePanel />
      </GridItem>

      <GridItem bg="blackAlpha.200">
        <Editors />
      </GridItem>

      <GridItem bg="blackAlpha.300">
        <Chat />
      </GridItem>
    </SimpleGrid>
  );

  return (
    <Authed>
      <RoomProvider>{layout}</RoomProvider>
    </Authed>
  );
};

export default RoomPage;
