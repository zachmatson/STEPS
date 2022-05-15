import React from "react";

export type PageLayoutProps = {
  bodyLeft: React.ReactNode;
  bodyRight: React.ReactNode;
  footer: React.ReactNode;
};

export const PageLayout = (props: PageLayoutProps) => (
  <div className="h-full w-full max-w-full 2xl:max-w-[1536px] mx-auto flex flex-col">
    <header className="flex-none">
      <Header />
    </header>
    <main className="h-full flex-1 overflow-y-auto scroll flex flex-col lg:flex-row">
      <div className="lg:w-1/3 pb-2 lg:pb-0 px-0">{props.bodyLeft}</div>
      <div className="lg:w-2/3 px-0">{props.bodyRight}</div>
    </main>
    <footer className="flex-none">{props.footer}</footer>
  </div>
);

const Header = () => (
  <div className="w-full py-3 lg:py-6 flex justify-center items-center border-gray-300 border-b-2 lg:border-0 text-3xl select-none cursor-default">
    STEPS Portal
  </div>
);
