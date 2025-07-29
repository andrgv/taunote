import React, { ReactNode } from 'react';

interface TopbarProps {
  title: string;
  children?: ReactNode;
}

const Topbar = ({
    title,
    children
}: TopbarProps) => {
    return (
        <header className="flex items-center justify-between px-8 py-4 border-b border-gray-700 bg-background z-10">
            <h1 className="text-2xl font-semibold text-white">{title}</h1>
            {children && (
                <div className="flex items-center space-x-4">
                    {children}
                </div>
            )}
        </header>
    );
}

export default Topbar;